extern crate orca;
extern crate regex;

use orca::App;
use regex::Regex;
use std::{thread, time};

fn main() {
    let reddit = App::new("orca_stream_example", "1.0", "/u/ognarb1").unwrap();

    // take the last posted comment as last viewed comment
    let mut last_viewed = reddit.get_recent_comments("all", Some(1), None).unwrap().nth(0).unwrap().id;

    //wait for new comments happening after last_viewed
    thread::sleep(time::Duration::from_millis(300));

    //only compile this one time
    let imgur_regexp = Regex::new(r"imgur.com/[a-zA-Z0-9]*");

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
                println!("id : {}", c.id);
            }
        }
        last_viewed = new_last_viewed.clone();
        thread::sleep(time::Duration::from_secs(5));
    }
}
