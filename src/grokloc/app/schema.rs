//! Schemas contains db schemas

#[allow(dead_code)]
pub static APP_CREATE_SCHEMA_SQLITE: &str = r#"
create table if not exists users (
       api_secret text unique not null,
       api_secret_digest text unique not null,
       id text unique not null,
       display_name text not null,
       display_name_digest text not null,
       email text not null,
       email_digest text not null,
       org text not null,
       password text not null,
       schema_version integer not null default 0,
       status integer not null,
       ctime integer,
       mtime integer,
       primary key (id));
-- STMT
create unique index if not exists users_email_org on users (email_digest, org);
-- STMT
create trigger if not exists users_ctime_trigger after insert on users
begin
        update users set
        ctime = strftime('%s','now'),
        mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create trigger if not exists users_mtime_trigger after update on users
begin
        update users set mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create table if not exists orgs (
       id text unique not null,
       name text unique not null,
       owner text not null,
       schema_version integer not null default 0,
       status integer not null,
       ctime integer,
       mtime integer,
       primary key (id));
-- STMT
create trigger if not exists orgs_ctime_trigger after insert on orgs
begin
        update orgs set
        ctime = strftime('%s','now'),
        mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create trigger if not exists orgs_mtime_trigger after update on orgs
begin
        update orgs set mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create table if not exists repositories (
       id text unique not null,
       name text not null,
       org text not null,
       path text not null,
       upstream text not null,
       schema_version integer not null default 0,
       status integer not null,
       ctime integer,
       mtime integer,
       primary key (id));
-- STMT
create unique index if not exists repositories_name_org on repositories (name, org);
-- STMT
create trigger if not exists repositories_ctime_trigger after insert on repositories
begin
        update repositories set
        ctime = strftime('%s','now'),
        mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create trigger if not exists repositories_mtime_trigger after update on repositories
begin
        update repositories set mtime = strftime('%s','now')
        where id = new.id;
end;
-- STMT
create table if not exists audit (
      id text unique not null,
      code integer not null,
      source text not null,
      source_id text not null,
      schema_version integer not null default 0,
      ctime integer,
      mtime integer,
      primary key (id));
-- STMT
create trigger if not exists audit_ctime_trigger after insert on audit
      begin
      update audit set
      ctime = strftime('%s','now'),
      mtime = strftime('%s','now')
      where id = new.id;
end;
-- STMT
create trigger if not exists audit_mtime_trigger after update on audit
      begin
      update audit set mtime = strftime('%s','now')
      where id = new.id;
end;
"#;

#[cfg(test)]
mod tests {
    use sqlx::{ Row };
    use crate::grokloc::err;
    use super::*;

    #[async_std::test]
    async fn schema_test_sqlite_create_schema() -> Result<(), sqlx::Error> {
        let pool: sqlx::SqlitePool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:").await?;
        sqlx::query(APP_CREATE_SCHEMA_SQLITE).execute(&pool).await?;
        let count_before0: i64 = sqlx::query_scalar("select count(*) as count from orgs").
            fetch_one(&pool).await?;
        assert_eq!(0, count_before0);
        // insert
        sqlx::query("insert into orgs (id,name,owner,schema_version,status) values (?,?,?,?,?)").
            bind("id0").
            bind("name0").
            bind("owner0").
            bind(0_i64).
            bind(0_i64).
            execute(&pool).await?;
        // duplicate insert
        let r = sqlx::query("insert into orgs (id,name,owner,schema_version,status) values (?,?,?,?,?)").
            bind("id0").
            bind("name0").
            bind("owner0").
            bind(0_i64).
            bind(0_i64).
            execute(&pool).await;
        match r {
            Err(err) => {
                assert!(err::sqlx_duplicate(&err));
            },
            _ => unreachable!(),
        }

        // read from cloned pool
        let read_pool = pool.clone();
        let count_after0: i64 = sqlx::query_scalar("select count(*) as count from orgs").
            fetch_one(&read_pool).await?;
        assert_eq!(1, count_after0);
        let row0 = sqlx::query("select ctime,mtime from orgs where id = ?").
            bind("id0").
            fetch_one(&read_pool).await?;
        assert_eq!(row0.get::<i64,_>(0), row0.get::<i64,_>(1));
        assert_ne!(row0.get::<i64,_>(0), 0);
        // not found
        let row1 = sqlx::query("select ctime,mtime from orgs where id = ?").
            bind("id1").
            fetch_one(&read_pool).await;
        match row1 {
            Err(sqlx::Error::RowNotFound) => (),
            _ => unreachable!(),
        }
        Ok(())
    }
}
