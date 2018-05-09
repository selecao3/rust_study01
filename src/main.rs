#![feature(plugin, decl_macro, custom_derive, const_fn)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serde_derive;
extern crate rocket_contrib;

mod static_files;
mod task;
mod db;
#[cfg(test)] mod tests;

//同ディレクトリ内にあるフォルダはpubをつけなくて良い

use rocket::Rocket;
use rocket::request::{Form, FlashMessage};
use rocket::response::{Flash, Redirect};
use rocket_contrib::Template;

use task::{Task, Todo};

#[derive(Debug, Serialize)]
//Serialize: データベースに配列データをそのまま保存したい時などに使う
//アトリビュートについて：　アトリビュートはstructだけに適用されるものではない。
// 例えば、ある関数の上に#[test]というアトリビュートが付くと、その関数はテストを実行した際に自動的に実行されるテスト関数になる。
struct Context<'a, 'b>{
    msg: Option<(&'a str, &'b str)>,
    tasks: Vec<Task>
}
//'aでライフタイムを無視する。ざっくり言うとどこからでも引数が受け取れる


impl<'a, 'b> Context<'a, 'b> {
    //struct Contextに対するimpl
    pub fn err(conn: &db::Conn, msg: &'a str) -> Context<'static, 'a> {
        Context{msg: Some(("error", msg)), tasks: Task::all(conn)}
    }

    pub fn raw(conn: &db::Conn, msg: Option<(&'a str, &'b str)>) -> Context<'a, 'b> {
        Context{msg: msg, tasks: Task::all(conn)}
    }
}

#[post("/", data = "<todo_form>")]
//アトリビュートpostにより、new関数はpost時に実行される関数に。
fn new(todo_form: Form<Todo>, conn: db::Conn) -> Flash<Redirect> {
    //rustではわざわざ返り値の型を書かなければならない。何故ならコンパイルエラー時に問題を分かりやすくするため.
    /*
    ・返り値について
    ややこしいけど、この場合のFlash<Redirect>という型で考える.
    まずFlashもRedirectも構造体である。
    下のif文を通ると漏れなくFlash::errorを通る。（定義を飛ぶと書いてあるけど）紆余曲折を経て、
    構造体Flashはパラメータを得る（name, message, consumed, innerの４つ ）
    次に構造体Redirectにパラメータが渡される...


    ざっくり言うと、裏ではstruct, impl, fnの三つが動いて複雑な構造を作っている。
    それに合わせて、返り値も決まっている
    この場合では、new関数は「入力された文字をサーバーにPOSTする」という結果を返す関数である。
    つまり、単純なString型などではなく複雑な型が返されないとおかしい。

    表現がおかしいけど、結局のところこの関数は「構造体Flash<Redirect>」に返している。
    struct Flash<Redirect>{
        hoge: aaa,
        hage: bbb,
        ...
    }
    struct内のメンバー、それぞれに任意の値が代入された状態
    */
    let todo = todo_form.into_inner();
    if todo.description.is_empty() {
        Flash::error(Redirect::to("/"), "Description cannot be empty.")
        //何も書かずにpostした時のエラー文
    } else if Task::insert(todo, &conn) {
        Flash::success(Redirect::to("/"), "Todo successfully added.")
    } else {
        Flash::error(Redirect::to("/"), "Whoops! The server failed.")
    }
}

#[put("/<id>")]
fn toggle(id: i32, conn: db::Conn) -> Result<Redirect, Template> {
    if Task::toggle_with_id(id, &conn) {
        Ok(Redirect::to("/"))
    } else {
        Err(Template::render("index", &Context::err(&conn, "Couldn't toggle task.")))
    }
}

/*
構造体Resultのメンバ、Ok()とErr()が渡される。
Okの中身はRedirectのメンバへ、Errの中身はTemplateのメンバへ渡され、処理される。
(より正確に言うと、各トレイトで処理されて、生成されたオブジェクトが各メンバに渡される。)
*/


#[delete("/<id>")]
fn delete(id: i32, conn: db::Conn) -> Result<Flash<Redirect>, Template> {
    if Task::delete_with_id(id, &conn) {
        Ok(Flash::success(Redirect::to("/"), "Todo was deleted."))
    } else {
        Err(Template::render("index", &Context::err(&conn, "Couldn't delete task.")))
    }
}

#[get("/")]
fn index(msg: Option<FlashMessage>, conn: db::Conn) -> Template {
    Template::render("index", &match msg {
        Some(ref msg) => Context::raw(&conn, Some((msg.name(), msg.msg()))),
        None => Context::raw(&conn, None),
    })
}

fn rocket() -> (Rocket, Option<db::Conn>) {
    let pool = db::init_pool();
    let conn = if cfg!(test) {
        Some(db::Conn(pool.get().expect("database connection for testing")))
    } else {
        None
    };

    let rocket = rocket::ignite()
        .manage(pool)
        .mount("/", routes![index, static_files::all])
        .mount("/todo/", routes![new, toggle, delete])
        .attach(Template::fairing());

    (rocket, conn)
    //返り値のタプル！
}

fn main() {
    rocket().0.launch();
}
