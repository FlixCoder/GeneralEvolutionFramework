extern crate rand;

use rand::Rng;

use std::cmp::Ordering;


/// Definition of necessary methods/properties for items to be optimized
pub trait GEF
{
	/// Uses itself and another item to breed a new item
	/// Can use averages or selection or both
	fn breed(&self, other:&Self) -> Self;
	/// Mutates itself
	/// Recommendation: use random reinitialization sometimes additional to random add/subtract
	fn mutate(&mut self);
	/// Evaluates itself and returns a score as 64-bit float. High is better.
	fn evaluate(&self) -> f64;
}

/// Evolutionary optimizer maximizing a given score on arbitrary items
pub struct Optimizer<T:GEF+Clone>
{
	items: Vec<(T,f64,u32)>, //items with the according score and survived evaluations
	//all necessary parameters are saved in learn_params and can be modified from the outside
	learn_params: (u32,u32,u32,f64,bool,u32), //population, survive, bad_survive, prob_mut, deterministic_selection, time to mean-avg over (minus 1)
}

impl<T:GEF+Clone> Optimizer<T>
{
	/// Create a new instance of an Optimizer with default parameters and an empty population. Use add_item before calling optimize
	pub fn new() -> Optimizer<T>
	{
		Optimizer {
					items: Vec::new(),
					learn_params: (200, 7, 3, 0.9, true, 0), //standard parameters
				}
	}
	
	
	/// Set population size (number of items to live after breeding; maximum amount of item generation)
	pub fn set_population(&mut self, pop:u32) -> &mut Self
	{
		if pop > self.learn_params.1 + self.learn_params.2
		{ //need more items than survive to generate new ones
			self.learn_params.0 = pop;
		}
		else
		{
			panic!("The population must be big enough to contain all surviving items and at least one free space for generation of new ones!");
		}
		
		self
	}
	
	/// Set number of best items to survive
	pub fn set_survive(&mut self, surv:u32) -> &mut Self
	{
		if surv >= 1
		{ //at least one must survive
			self.learn_params.1 = surv;
			
			if self.learn_params.0 <= self.learn_params.1 + self.learn_params.2
			{
				self.learn_params.0 = self.learn_params.1 + self.learn_params.2 + 1;
				println!("Population was increased to ensure a valid value!");
			}
		}
		else
		{
			panic!("At least one best item must survive!");
		}
		
		self
	}
	
	/// Set number of bad items to survive
	pub fn set_bad_survive(&mut self, bad_surv:u32) -> &mut Self
	{
		if bad_surv >= 1
		{ //at least one must survive
			self.learn_params.2 = bad_surv;
			
			if self.learn_params.0 <= self.learn_params.1 + self.learn_params.2
			{
				self.learn_params.0 = self.learn_params.1 + self.learn_params.2 + 1;
				println!("Population was increased to ensure a valid value!");
			}
		}
		else
		{
			panic!("At least one bad item must survive!");
		}
		
		self
	}
	
	/// Set probability of mutation
	pub fn set_prob_mutate(&mut self, prob_mut:f64) -> &mut Self
	{
		if prob_mut.is_nan() || prob_mut < 0.0 || prob_mut > 1.0
		{ //must be valid probability
			panic!("The given probability is not valid! Must be in [0.0, 1.0]");
		}
		else
		{
			self.learn_params.3 = prob_mut;
		}
		
		self
	}
	
	/// Set selection strategy
	/// deterministic = true => best x1 items and randomly chosen x2 bad items survive
	/// deterministic = false => stochastic - x1+x2 items are chosen, probability to survive decreases as cubic function
	pub fn set_selection_strat(&mut self, deterministic:bool) -> &mut Self
	{
		self.learn_params.4 = deterministic;
		
		self
	}
	
	/// Set number of iterations to build the mean average of the score over (standard is 1 => no avg)
	pub fn set_mean_avg(&mut self, n:u32) -> &mut Self
	{
		if n < 1
		{ //no score calculation possible with 0 iterations
			panic!("Mean-Avg over 0 iterations not possible!");
		}
		
		self.learn_params.5 = n - 1;
		
		self
	}
	
	
	/// Receive the best item from the population as clone
	/// Will panic if there is no item
	pub fn get_best(&self) -> T
	{
		self.items[0].0.clone()
	}
	
	/// Receive the best item from the population as reference
	/// Will panic if there is no item
	pub fn get_best_ref(&self) -> &T
	{
		&self.items[0].0
	}
	
	/// Receive the best item's score
	/// Will panic if there is no item
	pub fn get_score(&self) -> f64
	{
		self.items[0].1
	}
	
	/// Receive the worst item's score
	/// Will panic if there is no item
	pub fn get_worst_score(&self) -> f64
	{
		self.items.last().unwrap().1
	}
	
	/// Receive the whole population as reference
	pub fn get_items(&self) -> &Vec<(T,f64,u32)>
	{
		&self.items
	}
	
	/// Add an item to the population
	pub fn add_item(&mut self, item:T) -> &mut Self
	{
		let score = item.evaluate();
		let item_and_score = (item, score, 1);
		self.items.push(item_and_score);
		
		self
	}
	
	/// Do n iterations optimizing the items by: 1. breeding 2. mutating 3. evaluating 4. sorting 5. surviving 6. sorting
	/// Returns best score after last iteration
	pub fn optimize(&mut self, n:u32) -> f64
	{
		if self.items.len() < 1
		{ //no optimization possible without initial item
			panic!("No optimization possible without items!");
		}
		
		for _ in 0..n
		{
			self.populate() //mutation is randomly done on new items after breeding
				.evaluate()
				.sort()
				.survive()
				.sort();
		}
		
		self.get_score()
	}
	
	
	/// Populate the population and randomly mutate new items
	fn populate(&mut self) -> &mut Self
	{
		let mut rng = rand::thread_rng();
		let len = self.items.len();
		let missing = self.learn_params.0 as usize - len;
		
		for _ in 0..missing
		{
			//random two items to breed a new item
			let i1:usize = rng.gen::<usize>() % len;
			let i2:usize = rng.gen::<usize>() % len;
			let mut new_item = self.items[i1].0.breed(&self.items[i2].0);
			
			//mutate new item
			if rng.gen::<f64>() < self.learn_params.3
			{
				new_item.mutate();
			}
			
			self.items.push((new_item, 0.0, 0)); //add with score 0.0, will be evaluated soon
		}
		
		self
	}
	
	/// Evaluate all items in the population
	fn evaluate(&mut self) -> &mut Self
	{
		for item in &mut self.items
		{
			let score = (item.1 * item.2 as f64 + item.0.evaluate()) / (item.2 + 1) as f64;
			item.1 = score;
			item.2 = (item.2 + 1).min(self.learn_params.5);
		}
		
		self
	}
	
	/// Eliminates population, so that the best "survival" items and random "bad_survival" items survive
	fn survive(&mut self) -> &mut Self
	{
		if (self.learn_params.1 + self.learn_params.2) as usize >= self.items.len()
		{ //already done
			return self;
		}
		
		let mut rng = rand::thread_rng();
		
		if self.learn_params.4
		{ //deterministic selection
			let mut bad = self.items.split_off(self.learn_params.1 as usize); //best items kept
			
			//randomly select bad items
			for _ in 0..self.learn_params.2
			{
				if bad.is_empty() { return self; }
				let i:usize = rng.gen::<usize>() % bad.len();
				self.items.push(bad.swap_remove(i));
			}
		}
		else
		{ //stochastic selection
			let mut set = Vec::new(); //index set to keep items
			let size = (self.learn_params.1 + self.learn_params.2) as usize;
			
			//randomly select distinct items non-uniformly
			for _ in 0..size
			{
				let mut contained = true;
				let mut index:usize = 0;
				while contained
				{
					index = (rng.gen::<f64>().powf(3.0) * (size) as f64) as usize;
					contained = set.contains(&index);
				}
				set.push(index);
			}
			//efficiently keep only items indexed in the set
			set.sort_unstable();
			for i in 0..size
			{
				self.items.swap(i, set[i]);
			}
			self.items.truncate(size);
		}
		
		self
	}
	
	/// Sorts the items, so that the best is in front (index 0)
	fn sort(&mut self) -> &mut Self
	{ //best nets (high score) in front, bad and NaN nets at the end
		self.items.sort_by(|ref r1, ref r2| { //reverse partial cmp and check for NaN
				let r = (r2.1).partial_cmp(&r1.1);
				if r.is_some() { r.unwrap() }
				else
				{
					if r1.1.is_nan() { if r2.1.is_nan() { Ordering::Equal } else { Ordering::Greater } } else { Ordering::Less }
				}
			});
		
		self
	}
}

