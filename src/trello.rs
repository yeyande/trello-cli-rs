use reqwest;
use json;
use std::fs::File;
use std::io::Read;
use std::env;

#[allow(dead_code)]
pub struct TrelloRest {
    url: String,
    key: String,
    token: String
}

#[allow(dead_code)]
impl TrelloRest {
    pub fn new(url: String, key: String, token: String) -> TrelloRest {
        TrelloRest { url: url, key: key, token: token }
    }

    fn send_request(&self, path: String) -> json::JsonValue {
        let url = format!("{base_url}/{path}?key={key}&token={token}", base_url=self.url, key=self.key, token=self.token, path=path);
        let body = reqwest::blocking::get(&url).unwrap().text().unwrap();
        return json::parse(&body).unwrap();
    }

    pub fn get_boards(&self) -> json::JsonValue {
        return self.send_request("members/me/boards".to_string());
    }

    pub fn get_board(&self, id: String) -> json::JsonValue {
        return self.send_request(format!("boards/{}/lists", id));
    }

    pub fn get_cards(&self, id: String) -> json::JsonValue {
        return self.send_request(format!("lists/{}/cards", id));
    }
}

#[allow(dead_code)]
pub struct TrelloRestLocal {
    url: String,
    key: String,
    token: String
}

#[allow(dead_code)]
impl TrelloRestLocal {
    pub fn new(url: String, key: String, token: String) -> TrelloRestLocal {
        TrelloRestLocal { url: url, key: key, token: token }
    }

    fn read_file(path: String) -> json::JsonValue {
        info!("Reading file '{}'", path);
        let mut input = File::open(path).unwrap();
        let mut buffer = String::new();
        input.read_to_string(&mut buffer).unwrap();
        return json::parse(&buffer).unwrap();
    }

    pub fn get_boards(&self) -> json::JsonValue {
        return TrelloRestLocal::read_file(format!("{}/test_data/boards.json", env::current_dir().unwrap().display()));
    }

    pub fn get_board(&self, id: String) -> json::JsonValue {
        return TrelloRestLocal::read_file(format!("{}/test_data/{}/lists/lists.json", env::current_dir().unwrap().display(), id));
    }

    pub fn get_cards(&self, id: String) -> json::JsonValue {
        return TrelloRestLocal::read_file(format!("{}/test_data/{}.json", env::current_dir().unwrap().display(), id));
    }
}
