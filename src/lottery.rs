use crate::eventbrite::Profile;
use anyhow::{anyhow, Error};
use rand::{seq, thread_rng};
use rand::seq::SliceRandom;


pub fn draw(nb: i8, attendees: &Vec<Profile>) -> Result<Vec<Profile>, Error> {
    match nb {
        a if a < 0 => Err(anyhow!("InvalidDrawRequest")),
        0 => Ok(vec![]),
        _ => {
            let mut rng = thread_rng();
            Ok(attendees.choose_multiple(&mut rng,nb as usize)
                .cloned().collect())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        let actual = draw(1, attendees.as_ref());
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().as_slice(), vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }].as_slice());

        // TODO type error to reimplement with thiserror
        // let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        // let actual = draw(40, attendees.as_ref());
        // assert!(actual.is_err());
        // assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::NotEnoughtParticipant { asked: 40, existant: 1 });
        //
        // let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        // let actual = draw(-1, attendees.as_ref());
        // assert!(actual.is_err());
        // assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::InvalidDrawRequest {asked: -1});
        //
        // let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        // let actual = draw(-50, attendees.as_ref());
        // assert!(actual.is_err());
        // assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::InvalidDrawRequest {asked: -50});

        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }, Profile { first_name: "Fabien".to_string(), last_name: "Bernard".to_string() }];
        let actual = draw(0, &attendees);
        assert!(actual.is_ok());
        let vec : Vec<Profile> = Vec::new();
        assert_eq!(actual.unwrap().as_slice(), vec.as_slice());

    }
}