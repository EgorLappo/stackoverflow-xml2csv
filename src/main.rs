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

const BUF_SIZE: usize = 4096 * 2;


// stackoverflow post, keeping only relevant parts
// in particular, drop the body of the post
#[derive(Debug, Serialize)]
struct Post {
    id: usize,
    post_type: String,
    accepted_answer_id: Option<usize>,
    parent_id: Option<usize>,
    creation_date: String,
    score: i32,
    view_count: Option<usize>,
    owner_user_id: Option<usize>,
    title: Option<String>,
    tags: Option<String>,
    answer_count: Option<usize>,
    comment_count: Option<usize>,
}

fn process_post(e: &BytesStart) -> Result<Option<Post>, Box<dyn Error>> {

    let post_type_id: usize = e.try_get_attribute(b"PostTypeId")?.expect(&format!("post type not found in post {:?}", e)).unescape_value()?.to_string().parse()?;

    if post_type_id > 2 { return Ok(None)}; // skip answers, wiki, etc (https://meta.stackexchange.com/questions/99265/meaning-of-values-for-posttypeid-in-data-explorer-or-in-data-dump) 

    // parse id to usize
    let id: usize = e.try_get_attribute(b"Id")?
                   .expect(&format!("no post id found in post {:?}", e))
                   .unescape_value()?.to_string()
                   .parse()?;

    let accepted_answer_id: Option<usize> = e.try_get_attribute(b"AcceptedAnswerId")?
                                           .map(|x| x.unescape_value().unwrap().to_string()
                                                     .parse().expect("accepted answer id is not a number!"));

    let parent_id: Option<usize> = e.try_get_attribute(b"ParentId")?
                                      .map(|x| x.unescape_value().unwrap().to_string()
                                                .parse().expect("parent id is not a number!"));  

    let creation_date = e.try_get_attribute(b"CreationDate")?
                        .expect("no creation date found!")
                        .unescape_value()?.to_string();

    let score: i32 = e.try_get_attribute(b"Score")?
                     .expect(&format!("no score found in post {:?}", e))
                     .unescape_value()?.to_string()
                     .parse()?;

    let view_count: Option<usize> = e.try_get_attribute(b"ViewCount")?
                                    .map(|x| x.unescape_value().unwrap().to_string()
                                              .parse().expect("view count is not a number!"));

    let owner_user_id: Option<usize> = e.try_get_attribute(b"OwnerUserId")?
                                      .map(|x| x.unescape_value().unwrap().to_string()
                                                .parse().expect("owner user id is not a number!"));

    let title = e.try_get_attribute(b"Title")?
                .map(|x| x.unescape_value().unwrap().to_string());

    let tags = e.try_get_attribute(b"Tags")?
                .map(|x| x.unescape_value().unwrap().to_string());

    let answer_count: Option<usize> = e.try_get_attribute(b"AnswerCount")?
                                        .map(|x| x.unescape_value().unwrap().to_string()
                                                    .parse().expect("answer count is not a number!"));

    let comment_count: Option<usize> = e.try_get_attribute(b"CommentCount")?
                                        .map(|x| x.unescape_value().unwrap().to_string()
                                                    .parse().expect("comment count is not a number!"));
    
    if post_type_id == 1 {
        let post = Post {
            id,
            post_type: "question".to_string(),
            accepted_answer_id,
            parent_id,
            creation_date,
            score,
            view_count,
            owner_user_id,
            title,
            tags,
            answer_count,
            comment_count,
        };

        return Ok(Some(post))
    } else if post_type_id == 2 {
        let post = Post {
            id,
            post_type: "answer".to_string(),
            accepted_answer_id,
            parent_id,
            creation_date,
            score,
            view_count,
            owner_user_id,
            title,
            tags,
            answer_count,
            comment_count,
        };

        return Ok(Some(post))
    } else {
        return Ok(None)
    }
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
    println!("{} posts processed, {} posts written to csv.", post_count, processed_count);

    Ok(())
}
