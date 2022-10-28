use std::{
    env,
    error::Error,
    fs::File,
    io::BufReader,
};

use quick_xml::{
    events::Event,
    events::BytesStart,
    Reader
};
use serde::Serialize;
use csv;

const BUF_SIZE: usize = 4096; // 4kb at once

// stackoverflow post, keeping only relevant parts
// in particular, drop the body of the post
#[derive(Debug, Serialize)]
struct Post {
    id: usize,
    accepted_answer_id: Option<usize>,
    creation_date: String,
    score: i32,
    view_count: usize,
    owner_user_id: Option<usize>,
    title: String,
    tags: String,
    answer_count: usize,
    comment_count: usize,
}

fn process_post(e: &BytesStart) -> Result<Option<Post>, Box<dyn Error>> {

    let post_type_id: u8 = e.try_get_attribute(b"PostTypeId")?.expect("post type not found!").unescape_value()?.to_string().parse().expect("post type is not a number!");

    if post_type_id == 2 { return Ok(None)}; // skip answers

    // parse id to usize
    let id: usize = e.try_get_attribute(b"Id")?
                   .expect("no post id found!")
                   .unescape_value()?.to_string()
                   .parse().expect("post id is not a number!");
    let accepted_answer_id: Option<usize> = e.try_get_attribute(b"AcceptedAnswerId")?
                                           .map(|x| x.unescape_value().unwrap().to_string()
                                                     .parse().expect("post id is not a number!"));
    let creation_date = e.try_get_attribute(b"CreationDate")?
                        .expect("no creation date found!")
                        .unescape_value()?.to_string();
    let score: i32 = e.try_get_attribute(b"Score")?
                     .expect("no score found!")
                     .unescape_value()?.to_string()
                     .parse().expect("score is not a number!");
    let view_count: usize = e.try_get_attribute(b"ViewCount")?
                           .expect("no view count found!")
                           .unescape_value()?.to_string()
                           .parse().expect("view count is not a number!");
    let owner_user_id: Option<usize> = e.try_get_attribute(b"OwnerUserId")?
                                      .map(|x| x.unescape_value().unwrap().to_string()
                                                .parse().expect("owner user id is not a number!"));
    let title = e.try_get_attribute(b"Title")?
                    .expect("no title found!")
                    .unescape_value()?.to_string();
    let tags = e.try_get_attribute(b"Tags")?
                .expect("no tags found!")
                .unescape_value()?.to_string();
    let answer_count: usize = e.try_get_attribute(b"AnswerCount")?
                             .expect("no answer count found!")
                             .unescape_value()?.to_string()
                             .parse().expect("answer count is not a number!");
    let comment_count: usize = e.try_get_attribute(b"CommentCount")?
                              .expect("no comment count found!")
                              .unescape_value()?.to_string()
                              .parse().expect("comment count is not a number!");
    
    let post = Post {
        id,
        accepted_answer_id,
        creation_date,
        score,
        view_count,
        owner_user_id,
        title,
        tags,
        answer_count,
        comment_count,
    };

    Ok(Some(post))
}

// write the parsed Post to a csv file given a handle using csv and serde
fn write_post_to_csv(post: &Post, writer: &mut csv::Writer<File>) -> Result<(), Box<dyn Error>> {
    writer.serialize(post)?;
    Ok(())
}   

fn main() -> Result<(), Box<dyn Error>> {
    let input = env::args().nth(1).ok_or("no input filename provided")?;
    let output = env::args().nth(2).ok_or("no output filename provided")?;

    // deal with nput
    let f = File::open(&input).map_err(|e| format!("failed to open {}: {}", input, e))?;
    let xmlfile = BufReader::new(f);
    let mut xmlfile = Reader::from_reader(xmlfile);

    // deal with output
    let csvfile = File::create(output)?;
    let mut csvwriter = csv::Writer::from_writer(csvfile);

    let mut buf = Vec::with_capacity(BUF_SIZE);
    let mut post_count: usize = 0;
    let mut processed_count: usize = 0;

    loop {
        match xmlfile.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Empty(ref e) => {
                if e.name().as_ref() == b"row" {
                    post_count += 1;

                    let post = process_post(e).expect("error processing a post record");

                    if let Some(post) = post {
                        processed_count += 1;
                        write_post_to_csv(&post, &mut csvwriter)?
                    } 
                }
            }
            _ => continue,

        };
        buf.clear();
    }

    println!("Done!");
    println!("{} posts processed, {} questions written to csv.", post_count, processed_count);

    Ok(())
}
