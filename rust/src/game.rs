pub trait Game {
    fn update(&mut self);
    fn render(&self);
}

pub struct GameState {

}

impl Game for GameState {

}