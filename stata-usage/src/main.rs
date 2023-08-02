mod res {
    include!(concat!(env!("OUT_DIR"), "/res.rs"));
}

fn main() {
    println!("res::classes::CAT = {:#?}", res::classes::CAT);
    println!("res::classes::DOG = {:#?}", res::classes::DOG);
    println!("res::items::APPLE = {:#?}", res::items::APPLE);
    println!("res::items::IRON_SWORD = {:#?}", res::items::IRON_SWORD);
}
