// UCI implementation according to this spec: https://gist.github.com/DOBRO/2592c6dad754ba67e6dcaec8c90165bf
// Doesn't necessarily implement the full spec, only what is needed

pub trait UciReciever {
    /// Tell the engine to use UCI
    /// After recieving this command, the engine must identify itself with the "id" command
    /// and send the "options" command to tell the GUI which engine settings are available.
    /// Finally, the engine should send the "uciok" command to acknowledge the uci mode
    fn use_uci();

    /// Switch the debug mode of the engine on and off.
    /// In debug mode the engine should send additional infos to the GUI, e.g. with the "info string" command
    /// to help debugging, e.g. the commands that the engine has recieved.
    /// Should be off by default and but can always recieve the command to be turned on, even while thinking
    fn use_debug(debug: bool);

    /// Basically just need to reply with "readyok" always?
    fn is_ready();

    // fn set_option<T>(option_id: &str, value: Option<T>);
}

pub struct UciSender;

impl UciSender {
    pub fn id(name: &str, author: &str) {
        println!("id name {}", name);
        println!("id author {}", author);
    }

    pub fn acknowledge_uci() {
        println!("uciok");
    }

    pub fn engine_is_ready() {
        println!("readyok");
    }

    /// Send the best move according to the engine, in algebraic notation
    pub fn best_move(best_move: &str) {
        println!("bestmove {}", best_move);
    }
}

pub enum UciInfo {
    Depth,
    Time,
    Nodes,
    Pv,
    MultiPv,
    Score,
    Hashfull,
    Nps,
    Refutation,
    CurrentLine,
}
