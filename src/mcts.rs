use std::{
    cell::RefCell, collections::VecDeque, f32::consts::SQRT_2, fmt::Display, rc::Rc, sync::mpsc,
    thread, time::SystemTime,
};

use uuid::Uuid;

use crate::board::{Board, Move, Player, Util};

const C: f32 = SQRT_2;
const NUM_THREADS: usize = 16;
const SIMULATE_CUTOFF: usize = 82;
const HEURISTIC_WEIGHT: f32 = 0.1;

struct Node {
    state: Board,
    children: Vec<Rc<RefCell<Node>>>,
    prev_action: Option<Move>,
    u: f32,   // total utility
    n: usize, // total playous
}
impl Node {
    fn new_root(init_state: Board) -> Node {
        Node {
            state: init_state,
            children: Vec::new(),
            prev_action: None,
            u: 0.0,
            n: 0,
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Generate children of this node for every move, return an arbitary child
    fn expand(&mut self) -> Rc<RefCell<Node>> {
        // println!("Expanding node with prev_action: {:?}", self.prev_action);
        if self.is_leaf() {
            for m in self.state.actions() {
                let mut new_board = self.state.clone();
                new_board
                    .place(m)
                    .expect("can't do move when expanding node");
                let child = Node {
                    state: new_board,
                    children: Vec::new(),
                    prev_action: Some(m),
                    u: 0.0,
                    n: 0,
                };
                let child_ptr = Rc::new(RefCell::new(child));
                self.children.push(child_ptr);
            }
        }
        self.children[fastrand::usize(..self.children.len())].clone()
    }

    /// Play n game, return the result of that game
    fn simulate(&self, player: Player, n: usize) -> Util {
        // println!("Simulating node with prev_action: {:?}", self.prev_action);

        let (tx, rx) = mpsc::channel();
        for _ in 0..n {
            let mut simulated_state = self.state.clone();
            let tx_clone = tx.clone();
            let sim_job = move || {
                let mut util = simulated_state.utility(player);
                let mut num_moves_simulated = 0;
                while util.is_none() {
                    if num_moves_simulated > SIMULATE_CUTOFF {
                        util = Some(simulated_state.heuristic(player) * HEURISTIC_WEIGHT);
                        break;
                    }
                    simulated_state.place_random().unwrap();
                    util = simulated_state.utility(player);
                    num_moves_simulated += 1;
                }

                tx_clone.send(util.unwrap()).unwrap();
            };
            let _thread = thread::spawn(sim_job);
        }

        drop(tx);
        let mut total_util = 0.0;
        for util in rx {
            total_util += util;
        }

        total_util
    }

    fn update(&mut self, util: Util, n: usize) {
        // println!("Updating node with prev_action: {:?}", self.prev_action);
        self.n += n;
        self.u += util;
    }
}

pub struct SearchTree {
    root_node_ptr: Rc<RefCell<Node>>,
}
impl SearchTree {
    pub fn monte_carlo(&mut self, time_limit: f32) -> Move {
        let start_time = SystemTime::now();

        let player = self.root_node_ptr.borrow().state.player;
        let mut game_simulated = 0;
        while start_time.elapsed().unwrap().as_secs_f32() < time_limit {
            let mut path = self.select();
            // println!("path len {}", path.len());
            let util = {
                let leaf_ptr = path.last().expect("path is empty").clone();
                let mut leaf = leaf_ptr.borrow_mut();
                let node_to_simulate_ptr = if leaf.n == 0 {
                    drop(leaf);
                    leaf_ptr
                } else {
                    let child_ptr = leaf.expand();
                    path.push(child_ptr.clone());
                    child_ptr
                };
                let node_to_simulate = node_to_simulate_ptr.borrow();
                node_to_simulate.simulate(player, NUM_THREADS)
            };
            game_simulated += NUM_THREADS;
            SearchTree::back_propagation(path, util, NUM_THREADS);
        }
        println!("Games simulated: {}", game_simulated);
        let mut max_n = usize::MIN;
        let mut max_idx = None;
        let mut root_node = self.root_node_ptr.borrow_mut();

        for (i, child_ptr) in root_node.children.iter().enumerate() {
            let child = child_ptr.borrow();
            if child.n > max_n {
                max_idx = Some(i);
                max_n = child.n;
            }
        }
        let best_child_ptr = match max_idx {
            Some(mi) => root_node.children[mi].clone(),
            None => root_node.expand(),
        };
        let best_child = best_child_ptr.borrow();
        // let best_child = root_node.children[max_idx.expect("root node is leaf node")].borrow();
        best_child.prev_action.unwrap()
    }

    pub fn new(init_state: Board) -> SearchTree {
        SearchTree {
            root_node_ptr: Rc::new(RefCell::new(Node::new_root(init_state))),
        }
    }

    // Return the path from the root to the node that has no children yet
    fn select(&mut self) -> Vec<Rc<RefCell<Node>>> {
        let mut node_ptr = self.root_node_ptr.clone();
        let mut path = Vec::new();
        loop {
            let node_ptr_clone = node_ptr.clone();
            let node = node_ptr_clone.borrow(); // need node to borrow from clone to be able to
                                                // reassign node_ptr at the end
            path.push(node_ptr);

            if node.is_leaf() {
                return path;
            }

            // calculate children's ucb1
            let mut max_ucb1 = f32::MIN;
            let mut max_idx = None;
            for (i, child_ptr) in node.children.iter().enumerate() {
                let mut child = child_ptr.borrow_mut();
                let mut ucb1 = if child.n == 0 {
                    f32::INFINITY
                } else {
                    child.u as f32 / child.n as f32
                        + C * f32::sqrt(f32::ln(node.n as f32) / child.n as f32)
                };
                if fastrand::bool() && child.state.are_there_threats() {
                    ucb1 *= 1.0 + (fastrand::f32() * 0.25);
                }

                // println!(
                //     "ucb1 for node with prev_action: {:?} is {}",
                //     node.prev_action, ucb1
                // );

                if ucb1 > max_ucb1 {
                    max_idx = Some(i);
                    max_ucb1 = ucb1;
                }
            }

            node_ptr = node.children[max_idx.expect("Unable to find max ucb1 value")].clone();
        }
    }

    fn back_propagation(path: Vec<Rc<RefCell<Node>>>, util: Util, n: usize) {
        for node_ptr in path {
            let mut node = node_ptr.borrow_mut();
            node.update(util, n);
        }
    }

    pub fn apply_move(&mut self, m: Move) {
        let mut root_node = self.root_node_ptr.borrow_mut();
        if root_node.is_leaf() {
            root_node.expand();
        }
        let mut target_node = None;
        for child_ptr in root_node.children.iter() {
            let child = child_ptr.borrow();
            if child.prev_action.unwrap() == m {
                target_node = Some(child_ptr.clone());
                break;
            }
        }

        match target_node {
            None => {
                println!("Creating new tree");
                let mut new_init_board = root_node.state.clone();
                new_init_board.place(m).unwrap();
                drop(root_node);
                *self = SearchTree::new(new_init_board);
            }
            Some(node_ptr) => {
                drop(root_node);
                self.root_node_ptr = node_ptr;
            }
        }
    }
}
impl Display for SearchTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut q = VecDeque::new();
        q.push_back((self.root_node_ptr.clone(), None));
        while !q.is_empty() {
            let (node_ptr, parent_id) = q.pop_front().unwrap();
            let node = node_ptr.borrow();
            if let Some(id) = parent_id {
                write!(f, "parent: {id} ")?;
            }
            if let Some(m) = node.prev_action {
                write!(f, "m: {:?} ", m)?;
            }
            let this_id: String = Uuid::new_v4().to_string().chars().take(4).collect();
            writeln!(f, "n: {:<3} u: {:<3} id:{}", node.n, node.u, this_id)?;
            for child_ptr in node.children.iter() {
                q.push_back((child_ptr.clone(), Some(this_id.clone())));
            }
        }
        writeln!(f)
    }
}
