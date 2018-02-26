//! Models for managing languages

table! {
    languages (id) {
        id -> Integer,
        name -> VarChar,
    }
}

#[derive(Serialize, Queryable, Insertable, Debug)]
#[table_name = "languages"]
pub struct Language {
    pub id: i32,
    pub name: String,
}