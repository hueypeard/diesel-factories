#![allow(proc_macro_derive_resolution_fallback)]

#[macro_use]
extern crate diesel;

use diesel::{pg::PgConnection, prelude::*};
use diesel_factories::{Association, Factory};

mod schema {
    table! {
        users (id) {
            id -> Integer,
            name -> Text,
            age -> Integer,
            country_id -> Nullable<Integer>,
            home_city_id -> Nullable<Integer>,
            current_city_id -> Nullable<Integer>,
        }
    }

    table! {
        countries (identity) {
            identity -> Integer,
            name -> Text,
        }
    }

    table! {
        cities (id) {
            id -> Integer,
            name -> Text,
            team_association -> Text,
            association_label -> Text,
            country_id -> Integer,
        }
    }
}

#[derive(Queryable, Clone)]
struct User {
    pub id: i32,
    pub name: String,
    pub age: i32,
    pub country_id: Option<i32>,
    pub home_city_id: Option<i32>,
    pub current_city_id: Option<i32>,
}

#[derive(Queryable, Clone)]
struct Country {
    pub identity: i32,
    pub name: String,
}

#[derive(Queryable, Clone)]
struct City {
    pub id: i32,
    pub name: String,
    pub team_association: String,
    pub association_label: String,
    pub country_id: i32,
}

#[derive(Clone, Factory)]
#[factory(
    model = "User",
    table = "crate::schema::users",
    connection = "diesel::pg::PgConnection"
)]
struct UserFactory<'b> {
    pub name: String,
    pub age: i32,
    pub country: std::option::Option<diesel_factories::Association<'b, Country, CountryFactory>>,
    pub home_city: Option<diesel_factories::Association<'b, City, CityFactory<'b>>>,
    pub current_city: Option<Association<'b, City, CityFactory<'b>>>,
}

impl<'b> Default for UserFactory<'b> {
    fn default() -> Self {
        Self {
            name: "Bob".into(),
            age: 30,
            country: None,
            home_city: None,
            current_city: None,
        }
    }
}

#[derive(Clone, Factory)]
#[factory(
    model = "Country",
    table = "crate::schema::countries",
    id_name = "identity"
)]
struct CountryFactory {
    pub name: String,
}

impl Default for CountryFactory {
    fn default() -> Self {
        Self {
            name: "Denmark".into(),
        }
    }
}

#[derive(Clone, Factory)]
#[factory(model = "City", table = "crate::schema::cities")]
struct CityFactory<'b> {
    pub name: String,
    pub team_association: String,
    pub association_label: String,
    pub country: Association<'b, Country, CountryFactory>,
}

impl<'b> Default for CityFactory<'b> {
    fn default() -> Self {
        Self {
            name: "Copenhagen".into(),
            team_association: "teamfive".into(),
            association_label: "thebest".into(),
            country: Association::default(),
        }
    }
}

#[test]
fn insert_one_user() {
    let con = setup();

    let user = UserFactory::default().name("Alice").insert(&con);

    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(1, count_users(&con));
    assert_eq!(0, count_countries(&con));
}

#[test]
fn overriding_country() {
    let con = setup();

    let bob = UserFactory::default()
        .country(Some(CountryFactory::default().name("USA")))
        .insert(&con);

    let country = find_country_by_id(bob.country_id.unwrap(), &con);

    assert_eq!("USA", country.name);
    assert_eq!(1, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

#[test]
fn insert_two_users_sharing_country() {
    let con = setup();

    let country = CountryFactory::default().insert(&con);
    let bob = UserFactory::default().country(Some(&country)).insert(&con);
    let alice = UserFactory::default().country(Some(&country)).insert(&con);

    assert_eq!(bob.country_id, alice.country_id);
    assert_eq!(2, count_users(&con));
    assert_eq!(1, count_countries(&con));
}

fn setup() -> PgConnection {
    let database_url = "postgres://localhost/diesel_factories_test";
    let con = PgConnection::establish(&database_url).unwrap();
    con.begin_test_transaction().unwrap();
    con
}

fn count_users(con: &PgConnection) -> i64 {
    use crate::schema::users;
    use diesel::dsl::count_star;
    users::table.select(count_star()).first(con).unwrap()
}

fn count_countries(con: &PgConnection) -> i64 {
    use crate::schema::countries;
    use diesel::dsl::count_star;
    countries::table.select(count_star()).first(con).unwrap()
}

fn find_country_by_id(input: i32, con: &PgConnection) -> Country {
    use crate::schema::countries::dsl::*;
    countries
        .filter(identity.eq(&input))
        .first::<Country>(con)
        .unwrap()
}
