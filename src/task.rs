/*
ここではデータベースでの処理を記述している。
*/


use diesel;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

mod schema {
    table! {
        tasks {
            id -> Nullable<Integer>,
            description -> Text,
            completed -> Bool,
        }
    }
}
//mod: 上のmodキーワードよりschemaブロックで包まれたtable!マクロはモジュールになった。

use self::schema::tasks;
use self::schema::tasks::dsl::{tasks as all_tasks, completed as task_completed};

#[table_name="tasks"]
#[derive(Serialize, Queryable, Insertable, Debug, Clone)]
pub struct Task {
    pub id: Option<i32>,
    pub description: String,
    pub completed: bool
}

#[derive(FromForm)]
pub struct Todo {
    pub description: String,
}

impl Task {
    /*
    ここでは構造体Taskについての処理を書いている。
    例えば、insert関数がmain.rsで呼ばれると、処理が行われる(多分、POSTメソッドを実行する関数)
    insert関数から返された値はbool型として返される
    */
    pub fn all(conn: &SqliteConnection) -> Vec<Task> {
        all_tasks.order(tasks::id.desc()).load::<Task>(conn).unwrap()
    }

    pub fn insert(todo: Todo, conn: &SqliteConnection) -> bool {
        let t = Task { id: None, description: todo.description, completed: false};
        //メモ：completedをfalseにすると文字に横線が引かれる。
        diesel::insert_into(tasks::table).values(&t).execute(conn).is_ok()
    }

    pub fn toggle_with_id(id: i32, conn: &SqliteConnection) -> bool {
        let task = all_tasks.find(id).get_result::<Task>(conn);
        if task.is_err() {
            return false;
        }

        let new_status = !task.unwrap().completed;
        let updated_task = diesel::update(all_tasks.find(id));
        updated_task.set(task_completed.eq(new_status)).execute(conn).is_ok()
    }

    pub fn delete_with_id(id: i32, conn: &SqliteConnection) -> bool {
        diesel::delete(all_tasks.find(id)).execute(conn).is_ok()
    }
}
