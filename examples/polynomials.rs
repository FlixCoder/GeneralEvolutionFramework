extern crate genevofra;
extern crate rand;

use genevofra::*;
use rand::Rng;
use std::sync::Arc;

const PROB_DEGREE:f64 = 0.02; //probability to increase the degree of a polynomial function by 1
const PROB_NEW:f64 = 0.05; //probability to set to random values with same degree
const RANGE_NEW:f64 = 5.0; //size of interval to create random factors (-2.5 - 2.5)
const PROB_MOD:f64 = 0.9; //for every value: probability to modify
const RANGE_MOD:f64 = 1.0; //size of interval to modify factors (-0.5 - 0.5)

const REGULARIZATION:f64 = 0.5; //factor for additional error term (multiplied to degree)


//optimizing polynomial functions to fit to the points
fn main()
{
	let points = vec![	(0.0, 0.1),
						(1.0, 1.4),
						(2.0, 4.0),
						(3.0, 8.4),
						(4.0, 16.5)
					];
	let points = Arc::new(points);
	
	let mut opt = Optimizer::new();
	opt.set_population(100)
		//.set_survive(7)
		//.set_bad_survive(3)
		//.set_prob_mutate(0.9)
		//.set_selection_strat(Strat::Deterministic)
		//.set_mean_avg(1)
		.add_item(Polynome::new(points.clone())) //add two initial items, could be one, could be more
		.add_item(Polynome::new(points)); //but the more items, the stabler is learning (survive + bad_survive is a good idea)
	
	//train
	for i in 0..10
	{
		let n = 5;
		let score = opt.optimize(n);
		println!("Score after {:3} iterations: {}", (i + 1) * n, score);
	}
	
	//output function
	let fct = opt.get_best_ref();
	fct.print();
}


fn calc_f(factors:&Vec<f64>, x:f64) -> f64
{
	let mut result = factors[0];
	let mut current_pow = 1.0;
	for i in 1..factors.len()
	{
		current_pow *= x;
		result += factors[i] * current_pow;
	}
	result
}


#[derive(Clone)]
struct Polynome
{
	fct:Vec<f64>, //factors accordingly for index => x^index
	target:Arc<Vec<(f64,f64)>>, //points that should be matched by the polynomial function
}

impl Polynome
{
	pub fn new(points:Arc<Vec<(f64,f64)>>) -> Polynome
	{
		let mut vec = Vec::new();
		vec.push(0.0); //constant zero function with degree 0
		Polynome { fct: vec, target: points }
	}
	
	pub fn print(&self)
	{
		let mut str = String::new();
		for i in 0..self.fct.len()
		{
			str.push_str(&format!("{}x^{} + ", self.fct[i], i));
		}
		let tmp = str.len() - 3;
		str.truncate(tmp);
		println!("{}", str);
	}
}

impl GEF for Polynome
{
	fn breed(&self, other:&Self) -> Polynome
	{
		let mut rng = rand::thread_rng();
		let mut new = self.clone();
		
		//for every element
		for i in 0..new.fct.len()
		{ //randomly decide between averaging, choosing own or choosing other value
			if other.fct.len() <= i { break; } //account for different degrees
			
			let rnd = rng.gen::<f64>();
			if rnd < 0.333
			{ //select value from other
				new.fct[i] = other.fct[i];
			}
			else if rnd < 0.666
			{ //average values
				new.fct[i] = (new.fct[i] + other.fct[i]) / 2.0;
			}
			//else is not necessary, own value chosen
		}
		
		new
	}
	
	fn mutate(&mut self)
	{
		let mut rng = rand::thread_rng();
		
		if rng.gen::<f64>() < PROB_DEGREE
		{ //increase polynomial degree
			self.fct.push(0.0);
		}
		
		if rng.gen::<f64>() < PROB_NEW
		{ //set to random
			for i in 0..self.fct.len()
			{
				self.fct[i] = rng.gen::<f64>() * RANGE_NEW - RANGE_NEW / 2.0;
			}
		}
		
		//for every value
		for i in 0..self.fct.len()
		{ //randomly choose to modify value
			if rng.gen::<f64>() < PROB_MOD
			{ //modify randomly
				let delta = rng.gen::<f64>() * RANGE_MOD - RANGE_MOD / 2.0;
				self.fct[i] += delta;
			}
		}
	}
	
	//evaluate as inverted mean squared error to target (we want to minimize instead of maximize)
	fn evaluate(&self) -> f64
	{
		//calculate mean squared error
		let mut mse = 0.0;
		for point in self.target.iter()
		{
			let error = point.1 - calc_f(&self.fct, point.0);
			mse += error * error;
		}
		mse /= self.target.len() as f64;
		
		//add regularization
		let reg = REGULARIZATION * (self.fct.len() - 1) as f64; //regularization
		
		//return
		-(mse + reg)
	}
}

