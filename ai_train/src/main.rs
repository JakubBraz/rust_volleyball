use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::{Duration, Instant};
use neural_network_lib::neural_network::NeuralNetwork;

fn main() {
    let mut network = NeuralNetwork::new(&[12, 500, 6]);
    let network_copy = network.clone();
    let learning_step = 0.05;

    let duration = Duration::from_secs(5 * 60);
    let time = Instant::now();
    let mut epoch = 0;
    println!("Start learning, duration: {:?}", duration);
    while time.elapsed() < duration {
    // while epoch < 100 {
        epoch += 1;
        println!("epoch: {}, time: {}", epoch, time.elapsed().as_secs_f32());
        learn_full_set(&mut network, learning_step);
    }
    println!("Learning done.");

    let test_inp: [f32; 12] = [0.42217308282852, 2.46899032592773, -3.52752709388733, 2.11540174484253, 5.5617356300354, 0.59872335195541, 2.91825008392334, 0.0, 1.23920571804047, 2.40825033187866, -3.0, 2.40676236152649];
    println!("{:?}", test_inp);
    let test_inp: [f32; 12] = test_inp.map(|x| x / 10.0);
    println!("{:?}", test_inp);
    println!("Before {:?}", network_copy.process(&test_inp));
    println!("After {:?}", network.process(&test_inp));

    let serialized = network.serialize();
    fs::write("test_network", serialized).unwrap();
    println!("Network saved");
}

fn learn_full_set(network: &mut NeuralNetwork, learning_rate: f32) {
    // let data_path = "C:\\Users\\jakubbraz\\me\\programming\\rust\\rust_volleyball\\game\\volleyball\\learning_data\\";
    let data_path = "C:\\Users\\jakubbraz\\Downloads\\learning_data\\";
    for file_path in fs::read_dir(data_path).unwrap() {
        let file_path = file_path.unwrap();
        // println!("{:?}", file_path);
        let file = File::open(file_path.path()).unwrap();
        let mut reader = BufReader::new(file);
        let mut lines = reader.lines();
        loop {
            match lines.next() {
                None => break,
                Some(line) => {
                    let inputs = line.unwrap();
                    // println!("{}", inputs);
                    let s: String = inputs.chars().skip(1).take_while(|&x| x != ']' ).collect();
                    let inputs: Vec<f32> = s.split(',')
                        .map(|x| x.trim().parse::<f32>().unwrap())
                        .map(|x| x / 10.0)
                        .collect();
                    // println!("input {:?}", arr);
                    let output = lines.next().unwrap().unwrap();
                    let target = match output.as_str() {
                        "0" => [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
                        "1" => [0.0, 1.0, 0.0, 0.0, 0.0, 0.0],
                        "2" => [0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
                        "3" => [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
                        "4" => [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                        "5" => [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
                        _ => unreachable!("Unexpected {}", output),
                    };

                    // ignore 'no action'
                    if output != "0" {
                        network.training_step(&inputs, &target, learning_rate)
                    }
                }
            }
        }
    }
}
