use eventbrite::Profile;
use failure::Error;
use rand::{seq, thread_rng};

#[derive(Debug, Fail, PartialEq)]
enum DrawError {
    #[fail(display = "Invalid draw request (asked : {})", asked)]
    InvalidDrawRequest {
        asked: i8
    },
    #[fail(display = "Not enough participants (asked: {}, existing: {})", asked, existant)]
    NotEnoughtParticipant {
        asked: i8,
        existant: usize,
    }
}

pub fn draw(nb: i8, attendees: &Vec<Profile>) -> Result<Vec<&Profile>, Error> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw() {
        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        let actual = draw(1, attendees.as_ref());
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().as_slice(), vec![&Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }].as_slice());

        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        let actual = draw(40, attendees.as_ref());
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::NotEnoughtParticipant { asked: 40, existant: 1 });

        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        let actual = draw(-1, attendees.as_ref());
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::InvalidDrawRequest {asked: -1});

        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }];
        let actual = draw(-50, attendees.as_ref());
        assert!(actual.is_err());
        assert_eq!(actual.unwrap_err().downcast::<DrawError>().unwrap(), DrawError::InvalidDrawRequest {asked: -50});

        let attendees = vec![Profile { first_name: "Francois".to_string(), last_name: "Teychene".to_string() }, Profile { first_name: "Fabien".to_string(), last_name: "Bernard".to_string() }];
        let actual = draw(0, &attendees);
        assert!(actual.is_ok());
        let vec : Vec<&Profile> = Vec::new();
        assert_eq!(actual.unwrap().as_slice(), vec.as_slice());

    }
}