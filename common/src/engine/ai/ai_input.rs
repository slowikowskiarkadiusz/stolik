use crate::engine::{ai::neat_genome::NeatGenome, input::input::Input};

pub trait AiInput: Input + Send {
    fn new(player: usize) -> Box<dyn AiInput + Send>
    where
        Self: Sized;

    fn set_genome(&mut self, genome: NeatGenome);

    fn get_genome(&self) -> &NeatGenome;
}
