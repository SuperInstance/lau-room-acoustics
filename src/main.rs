use lau_room_acoustics::*;

fn main() {
    let room = AcousticRoom {
        id: "demo-room".into(),
        dimensions: (5.0, 4.0, 3.0),
        absorption: 0.3,
        temperature: 20.0,
    };
    let engine = RoomAcousticsEngine::new(room);
    let analysis = engine.full_analysis();
    println!("{}", analysis.summary());
}
