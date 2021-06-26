use email_parser::prelude::*;
use std::fs::*;

#[ignore]
#[test]
fn enron() {
    let mut ok = 0;
    let mut tot = 0;
    'main: for dir in read_dir("/home/mubelotix/Downloads/enron_mail_20150507/maildir").unwrap() {
        if let Ok(dir) = dir {
            if let Ok(entries) = read_dir(dir.path()) {
                for dir in entries {
                    if let Ok(dir) = dir {
                        if let Ok(entries) = read_dir(dir.path()) {
                            for dir in entries {
                                if let Ok(file) = dir {
                                    if file.file_type().map(|f| f.is_file()).ok() == Some(true) {
                                        if let Ok(content) = read(file.path()) {
                                            let result = Email::parse(&content);

                                            tot += 1;
                                            match result {
                                                Ok(_email) => ok += 1,
                                                Err(e) => println!("{} at {:?}", e, file.path()),
                                            }

                                            if tot >= 500000 {
                                                break 'main;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    println!("{}/{}", ok, tot)
}

#[cfg(feature = "mime")]
#[test]
fn my_emails() {
    let mut ok = 0;
    let mut tot = 0;
    'main: for file in read_dir("/home/mubelotix/Downloads/emails").unwrap() {
        if let Ok(file) = file {
            if file.file_type().map(|f| f.is_file()).ok() == Some(true) {
                if let Ok(mut content) = read(file.path()) {
                    let mut i = 0;
                    loop {
                        match content.get(i) {
                            None => break,
                            Some(b'\n') => match content.get(i.saturating_sub(1)) {
                                Some(b'\r') => (),
                                Some(_) => content.insert(i, b'\r'),
                                None => (),
                            },
                            _ => (),
                        }
                        i += 1;
                    }

                    let result = Email::parse(&content);

                    tot += 1;
                    match result {
                        Ok(email) => {
                            println!("{:?}", email.subject);
                            ok += 1
                        }
                        Err(e) => println!("{} at {:?}", e, file.path()),
                    }

                    if tot >= 500000 {
                        break 'main;
                    }
                }
            }
        }
    }
    assert_eq!(ok, 332);
    println!("{}/{}", ok, tot)
}
