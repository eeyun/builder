use std::io::Write;
use std::ops::Deref;
use std::str::{self, FromStr};

use protobuf;
use protocol::originsrv::{OriginPackage, OriginPackageIdent, OriginPackageVisibility};

use chrono::NaiveDateTime;
use diesel;
use diesel::deserialize::{self, FromSql};
use diesel::pg::{Pg, PgConnection};
use diesel::result::QueryResult;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::sql_types::{Array, BigInt, SmallInt, Text};
use diesel::RunQueryDsl;

use super::db_id_format;
use hab_core;
use hab_core::package::{FromArchive, PackageArchive, PackageIdent, PackageTarget};
use schema::package::*;

#[derive(Debug, Serialize, Deserialize, QueryableByName, Clone)]
#[table_name = "origin_packages"]
pub struct Package {
    #[serde(with = "db_id_format")]
    pub id: i64,
    #[serde(with = "db_id_format")]
    pub origin_id: i64,
    #[serde(with = "db_id_format")]
    pub owner_id: i64,
    pub name: String,
    pub ident: BuilderPackageIdent,
    pub ident_array: Vec<String>,
    pub checksum: String,
    pub manifest: String,
    pub config: String,
    pub target: BuilderPackageTarget,
    pub deps: Vec<BuilderPackageIdent>,
    pub tdeps: Vec<BuilderPackageIdent>,
    pub exposes: Vec<i16>,
    pub visibility: PackageVisibility,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NewPackage {
    #[serde(with = "db_id_format")]
    pub origin_id: i64,
    #[serde(with = "db_id_format")]
    pub owner_id: i64,
    pub name: String,
    pub ident: BuilderPackageIdent,
    pub checksum: String,
    pub manifest: String,
    pub config: String,
    pub target: BuilderPackageTarget,
    pub deps: Vec<BuilderPackageIdent>,
    pub tdeps: Vec<BuilderPackageIdent>,
    pub exposes: Vec<i16>,
    pub visibility: PackageVisibility,
}

#[derive(Debug)]
pub struct GetLatestPackage {
    pub ident: BuilderPackageIdent,
    pub target: BuilderPackageTarget,
    pub visibility: Vec<PackageVisibility>,
}

pub struct GetPackage {
    pub ident: BuilderPackageIdent,
    pub visibility: Vec<PackageVisibility>,
}

#[derive(DbEnum, Debug, Serialize, Deserialize, Clone, ToSql, FromSql)]
#[PgType = "origin_package_visibility"]
#[postgres(name = "origin_package_visibility")]
pub enum PackageVisibility {
    #[postgres(name = "public")]
    Public,
    #[postgres(name = "private")]
    Private,
    #[postgres(name = "hidden")]
    Hidden,
}

impl Package {
    pub fn get(req: GetPackage, conn: &PgConnection) -> QueryResult<Package> {
        diesel::sql_query("select * from get_origin_package_v5($1, $2)")
            .bind::<Text, _>(req.ident)
            .bind::<Array<PackageVisibilityMapping>, _>(req.visibility)
            .get_result(conn)
    }

    pub fn get_latest(req: GetLatestPackage, conn: &PgConnection) -> QueryResult<Package> {
        debug!("HERE HERE HERE: {:?}", req);
        diesel::sql_query("select * from get_origin_package_latest_v6($1, $2, $3)")
            .bind::<Text, _>(req.ident)
            .bind::<Text, _>(req.target)
            .bind::<Array<PackageVisibilityMapping>, _>(req.visibility)
            .get_result(conn)
    }

    pub fn create(package: NewPackage, conn: &PgConnection) -> QueryResult<Package> {
        diesel::sql_query("SELECT * FROM insert_origin_package_v5($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)")
            .bind::<BigInt,_>(package.origin_id)
            .bind::<BigInt,_>(package.owner_id)
            .bind::<Text,_>(package.name)
            .bind::<Text,_>(package.ident)
            .bind::<Text,_>(package.checksum)
            .bind::<Text,_>(package.manifest)
            .bind::<Text,_>(package.config)
            .bind::<Text,_>(package.target)
            .bind::<Array<Text>,_>(package.deps)
            .bind::<Array<Text>,_>(package.tdeps)
            .bind::<Array<SmallInt>,_>(package.exposes)
            .bind::<PackageVisibilityMapping,_>(package.visibility)
        .get_result(conn)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuilderPackageIdent(pub PackageIdent);

impl FromSql<Text, Pg> for BuilderPackageIdent {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match bytes {
            Some(text) => Ok(BuilderPackageIdent(
                PackageIdent::from_str(str::from_utf8(text).unwrap()).unwrap(),
            )),
            None => Ok(BuilderPackageIdent(PackageIdent::default())),
        }
    }
}

impl ToSql<Text, Pg> for BuilderPackageIdent {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write_all(self.to_string().as_bytes())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl Into<PackageIdent> for BuilderPackageIdent {
    fn into(self) -> PackageIdent {
        self.0
    }
}

impl Deref for BuilderPackageIdent {
    type Target = PackageIdent;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuilderPackageTarget(pub PackageTarget);

impl FromSql<Text, Pg> for BuilderPackageTarget {
    fn from_sql(bytes: Option<&[u8]>) -> deserialize::Result<Self> {
        match bytes {
            Some(text) => Ok(BuilderPackageTarget(
                PackageTarget::from_str(str::from_utf8(text).unwrap()).unwrap(),
            )),
            None => Ok(BuilderPackageTarget(
                PackageTarget::from_str("x86_64").unwrap(),
            )),
        }
    }
}

impl ToSql<Text, Pg> for BuilderPackageTarget {
    fn to_sql<W: Write>(&self, out: &mut Output<W, Pg>) -> serialize::Result {
        out.write_all(self.to_string().as_bytes())
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl Deref for BuilderPackageTarget {
    type Target = PackageTarget;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromArchive for NewPackage {
    type Error = hab_core::Error;

    fn from_archive(archive: &mut PackageArchive) -> hab_core::Result<Self> {
        let ident = match archive.ident() {
            Ok(value) => BuilderPackageIdent(value),
            Err(e) => return Err(hab_core::Error::from(e)),
        };

        let config = match archive.config()? {
            Some(config) => config,
            None => String::from(""),
        };

        let exposes = archive
            .exposes()?
            .into_iter()
            .map(|e| e as i16)
            .collect::<Vec<i16>>();

        let deps = archive
            .deps()?
            .into_iter()
            .map(|d| BuilderPackageIdent(d))
            .collect::<Vec<BuilderPackageIdent>>();

        let tdeps = archive
            .tdeps()?
            .into_iter()
            .map(|d| BuilderPackageIdent(d))
            .collect::<Vec<BuilderPackageIdent>>();

        // Some of the values here are made up because they are required in the db but not
        // necessarially requred for a valid package
        Ok(NewPackage {
            ident: ident.clone(),
            manifest: archive.manifest()?,
            target: BuilderPackageTarget(archive.target()?),
            deps: deps,
            tdeps: tdeps,
            exposes: exposes,
            config: config,
            checksum: archive.checksum()?,
            name: ident.name.to_string(),
            origin_id: 999999999999,
            owner_id: 999999999999,
            visibility: PackageVisibility::Public,
        })
    }
}

// TED TODO: PROTOCLEANUP Remove everything below when the protos are gone
impl From<OriginPackageVisibility> for PackageVisibility {
    fn from(value: OriginPackageVisibility) -> PackageVisibility {
        match value {
            OriginPackageVisibility::Hidden => PackageVisibility::Hidden,
            OriginPackageVisibility::Private => PackageVisibility::Private,
            _ => PackageVisibility::Public,
        }
    }
}

impl Into<OriginPackageVisibility> for PackageVisibility {
    fn into(self) -> OriginPackageVisibility {
        match self {
            PackageVisibility::Hidden => OriginPackageVisibility::Hidden,
            PackageVisibility::Private => OriginPackageVisibility::Private,
            _ => OriginPackageVisibility::Public,
        }
    }
}

impl Into<OriginPackage> for Package {
    fn into(self) -> OriginPackage {
        let exposes = self
            .exposes
            .into_iter()
            .map(|e| e as u32)
            .collect::<Vec<u32>>();

        let mut op = OriginPackage::new();
        let ident = &*self.ident;
        op.set_id(self.id as u64);
        op.set_ident(OriginPackageIdent::from(ident.clone()));
        op.set_manifest(self.manifest);
        op.set_target(self.target.to_string());
        op.set_deps(into_idents(self.deps));
        op.set_tdeps(into_idents(self.tdeps));
        op.set_exposes(exposes);
        op.set_config(self.config);
        op.set_checksum(self.checksum);
        op.set_origin_id(self.origin_id as u64);
        op.set_owner_id(self.owner_id as u64);
        op.set_visibility(self.visibility.into());
        op
    }
}

impl Into<OriginPackageIdent> for BuilderPackageIdent {
    fn into(self) -> OriginPackageIdent {
        self.0.into()
    }
}

fn into_idents(column: Vec<BuilderPackageIdent>) -> protobuf::RepeatedField<OriginPackageIdent> {
    let mut idents = protobuf::RepeatedField::new();
    for ident in column {
        idents.push(ident.into());
    }
    idents
}