







// pub struct Entity {
//     tag: String,
//     components: HashMap<String, Box<dyn Component>>,
//     on_tick: Option<fn(&Self, f64) -> ()>,
// }

// impl Entity {
//     pub fn empty<S>(tag: S) -> Self
//     where
//         S: Into<String>,
//     {
//         Self {
//             tag: tag.into(),
//             components: HashMap::new(),
//             on_tick: Some(Self::on_tick),
//         }
//     }

//     pub fn get_on_tick(&self) -> Option<fn(&Self, f64) -> ()> {
//         self.on_tick
//     }

//     pub fn get_tag(&self) -> &String {
//         &self.tag
//     }
// }
