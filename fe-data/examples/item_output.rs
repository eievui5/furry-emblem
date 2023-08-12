use fe_data::*;

fn main() {
	let example_item = Item {
		name: String::from("Iron Sword"),
		ty: ItemType::Weapon(WeaponItem {
			damage: 5,
			weight: 5,
			..Default::default()
		}),
		..Default::default()
	};

	println!("{}", toml::to_string(&example_item).unwrap());
}
