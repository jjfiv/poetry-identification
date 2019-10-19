CREATE TABLE IF NOT EXISTS pages (
    id integer primary key autoincrement,
    book text not NULL,
    page integer not NULL,
    content blob not NULL
);