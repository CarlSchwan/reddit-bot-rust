extern crate orca;
extern crate regex;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use orca::App;
use orca::data::{Comment};
use std::{thread, time};
use std::collections::HashMap;
use std::fmt;

static UPDATE_INTERVAL_IN_SECONDS : u64 = 3;
static TESTING : bool = true;

fn main() {
    let reddit = App::new("orca_stream_example", "1.0", "/u/ognarb1").unwrap();
    print!("{}", analyse_last_n_comments(1000, reddit));
}

struct DatabaseEntry (HashMap<String, i64>, i64);

struct Database (HashMap<String, DatabaseEntry>, i64);

impl Database {
    fn sort(&self) -> Vec<(i64, &String)> {
        let mut h = BinaryHeap::new();
        for (word, entry) in self.0.iter() {
            h.push((entry.1, word));
        }
        h.into_sorted_vec()
    }
    fn new() -> Database {
        Database(HashMap::new(), 0)
    }
}

impl fmt::Display for Database {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        for (occurences, word) in self.sort() {
            write!(f, "{} times {}\n", occurences, word);
        }
        write!(f, "Analysed {} Words!\n", self.1)
    }
}

fn between(c:&char,  a: char, b: char) -> bool {
    *c >= a && *c <= b
}

fn addWordToDatabase <'a> (word: String, subreddit:&String, mut database:Database) -> Database {
    let mut sub = String::new();
    sub.clone_from(subreddit);
    let clean_word = String::from(word.trim_matches( | c | !(between(&c,'a','z') || between(&c,'A', 'Z') || between(&c, '0', '9'))).to_lowercase());
    //add databaseEntry if not existing
    {
        let entry = database.0.entry(clean_word).or_insert(DatabaseEntry (HashMap::new(), 0));
        *entry.0.entry(sub).or_insert(0) += 1;
        entry.1 += 1;
    }
    database
}

fn addToDatabase (comment: Comment, data:Database) -> Database {
    comment.body.split_whitespace().fold(data, | mut data, word | { 
        data.1 += 1;
        addWordToDatabase(String::from(word), &comment.subreddit, data) 
    })
}

fn analyse_last_n_comments(n: usize, reddit:App) -> Database {
    let mut data = Database::new();
    for comment in reddit.create_comment_stream("all").take(n) {
        data = addToDatabase(comment, data);
    }
    data
}

fn polling(reddit:App) {

    // take the last posted comment as last viewed comment
    let mut last_viewed = reddit.get_recent_comments("all", Some(1), None).unwrap().nth(0).unwrap().id;

    //wait for new comments happening after last_viewed
    thread::sleep(time::Duration::from_millis(300));

    let mut h = Database::new();
    
    //only compile this one time
    //let imgur_regexp = Regex::new(r"imgur.com/[a-zA-Z0-9]*");

    //polling for new comments
    loop {
        // get update from reddit
        let mut comments = reddit.create_comment_stream("all");
        let new_last_viewed = comments.nth(0).unwrap().id;
        // cannot assign to last_viewed as long as new_comments exists
        {
            //every comment in new_comments is garanteed to be new and not already processed by
            //this program
            let new_comments = comments.take_while( |x| x.id != last_viewed);
            for c in new_comments {
                //println!("id : {}", c.id);
                h = addToDatabase(c, h);
            }
        }
        last_viewed = new_last_viewed.clone();
        if TESTING {
            break;
        }
        thread::sleep(time::Duration::from_secs(UPDATE_INTERVAL_IN_SECONDS));
    }
    print!("Starting to print...");
    print!("{}", h);
}
