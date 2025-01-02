use std::fs;

use rusqlite::{params, Connection};

use crate::{config::Config, pantry, types::PackageReq};

pub fn cache(config: &Config) -> Result<Connection, Box<dyn std::error::Error>> {
    let path = config.pantry_dir.parent().unwrap().join("pantry.db");

    // Remove the existing database file
    if path.exists() {
        fs::remove_file(&path)?;
    }

    // Set up SQLite connection
    let mut conn = Connection::open(&path)?;

    conn.execute_batch(
        "
    PRAGMA synchronous = OFF;
    PRAGMA journal_mode = MEMORY;
    PRAGMA temp_store = MEMORY;
    DROP TABLE IF EXISTS provides;
    DROP TABLE IF EXISTS dependencies;
    DROP TABLE IF EXISTS companions;
    DROP TABLE IF EXISTS runtime_env;
    CREATE TABLE provides (
        project TEXT,
        program TEXT
    );
    CREATE TABLE dependencies (
        project TEXT,
        pkgspec TEXT
    );
    CREATE TABLE companions (
        project TEXT,
        pkgspec TEXT
    );
    CREATE TABLE runtime_env (
        project TEXT,
        envline TEXT
    );
    CREATE INDEX idx_project ON provides(project);
    CREATE INDEX idx_program ON provides(program);
    CREATE INDEX idx_project_dependencies ON dependencies(project);
    CREATE INDEX idx_project_companions ON companions(project);
    ",
    )?;

    let tx = conn.transaction()?;

    for pkg in pantry::ls(&config) {
        for program in pkg.programs {
            tx.execute(
                "INSERT INTO provides (project, program) VALUES (?1, ?2);",
                params![pkg.project, program],
            )?;
        }

        for dep in pkg.deps {
            tx.execute(
                "INSERT INTO dependencies (project, pkgspec) VALUES (?1, ?2);",
                params![pkg.project, dep.to_string()],
            )?;
        }

        for companion in pkg.companions {
            tx.execute(
                "INSERT INTO companions (project, pkgspec) VALUES (?1, ?2);",
                params![pkg.project, companion],
            )?;
        }

        for (key, value) in pkg.env {
            tx.execute(
                "INSERT INTO runtime_env (project, envline) VALUES (?1, ?2);",
                params![pkg.project, format!("{}={}", key, value)],
            )?;
        }
    }

    tx.commit()?;

    Ok(conn)
}

pub fn deps_for_project(
    project: &String,
    conn: &Connection,
) -> Result<Vec<PackageReq>, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT pkgspec FROM dependencies WHERE project = ?1")?;
    let rv = stmt.query_map(params![project], |row| {
        let pkgspec = row.get(0)?;
        let pkgrq = PackageReq::parse(pkgspec).unwrap(); //FIXME error handling
        Ok(pkgrq)
    })?;
    Ok(rv.collect::<Result<Vec<_>, _>>()?)
}

pub fn which(cmd: &String, conn: &Connection) -> Result<String, Box<dyn std::error::Error>> {
    let mut stmt = conn.prepare("SELECT project FROM provides WHERE program = ?1")?;
    let mut rows = stmt.query(params![cmd])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Err("No project found for the given command".into())
    }
}

// converts programs to projects and leaves projects
pub fn convert(
    args: Vec<String>,
    conn: &Connection,
) -> Result<Vec<PackageReq>, Box<dyn std::error::Error>> {
    let mut pkgs = vec![];

    for arg in args {
        let mut pkg = PackageReq::parse(arg)?;
        if let Ok(project) = which(&pkg.project, &conn) {
            pkg.project = project;
            pkgs.push(pkg);
        } else {
            pkgs.push(pkg);
        }
    }

    Ok(pkgs)
}
