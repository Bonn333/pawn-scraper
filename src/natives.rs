use samp_sdk::types::Cell;
use samp_sdk::amx::{AmxResult, AMX};
use scraper::{Html,Selector};
use minihttp::request::Request;

pub trait Natives {
	fn parse_document(&mut self,_:&AMX,document:String) -> AmxResult<Cell>;
	fn parse_document_by_response(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>;
	fn parse_selector(&mut self,_:&AMX,string:String) -> AmxResult<Cell>;
	fn http_request(&mut self,_:&AMX,url:String) -> AmxResult<Cell>;
	fn http_request_threaded(&mut self,_:&AMX,playerid:usize,callback:String,url:String) -> AmxResult<Cell>;
	fn get_nth_element_name(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,string:&mut Cell,size:usize) -> AmxResult<Cell>;
	fn get_nth_element_text(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,string:&mut Cell,size:usize) -> AmxResult<Cell>;
	fn get_nth_element_attr_value(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,attr:String,string:&mut Cell,size:usize) -> AmxResult<Cell>;
	fn delete_html_instance(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>;
	fn delete_selector_instance(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>;
	fn delete_response_cache(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>;	
}

impl Natives for super::PawnScraper{
	fn parse_document(&mut self,_:&AMX,document:String) -> AmxResult<Cell> {
		let parsed_data = Html::parse_document(&document);
		self.html_instance.insert(self.html_context_id,parsed_data);
		self.html_context_id += 1;
		Ok(self.html_context_id  as Cell -1)
	}

	fn parse_document_by_response(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>{
		if id > self.response_context_id {
			Ok(-1)
		}else{
			let response_data = self.response_cache.get(&id);
			if response_data == None{
				Ok(-1)
			}else{
				let parsed_data = Html::parse_document(&response_data.unwrap());
				self.html_instance.insert(self.html_context_id,parsed_data);
				self.html_context_id += 1;
				Ok(self.html_context_id  as Cell -1)
			}
		}
	}

	fn parse_selector(&mut self,_:&AMX,string:String) -> AmxResult<Cell> {
		match Selector::parse(&string){
			Ok(selector) => {
				self.selectors.insert(self.selector_context_id,selector);
				self.selector_context_id += 1;
				Ok(self.selector_context_id as Cell -1)
			}
			Err(err) =>{
				log!("**[PawnScraper] Failed parsing selector {:?}",err);
				Ok(-1)
			}
		}
	}

	fn get_nth_element_text(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,string:&mut Cell,size:usize) -> AmxResult<Cell>{
		if !self.html_instance.contains_key(&docid) || !self.selectors.contains_key(&selectorid){
			log!("**[PawnScraper] Invalid html instances passed docid {:?},selectorid {:?}",docid,selectorid);
			Ok(-1)
		}else{
			let html = &self.html_instance.get(&docid).unwrap();
			let selector = &self.selectors.get(&selectorid).unwrap();
			let nth_element = html.select(selector).nth(idx);
			if nth_element == None{
				Ok(0)
			}else{
				let element_text_iter = nth_element.unwrap().text();
				let mut full_text:String = String::new();
				for i in element_text_iter{
					full_text += i;
				}
				let text_encoded = samp_sdk::cp1251::encode(&full_text).unwrap();
				set_string!(text_encoded,string,size);
				Ok(1)
			}
		}
	}

	fn get_nth_element_name(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,string:&mut Cell,size:usize) -> AmxResult<Cell>{
		if !self.html_instance.contains_key(&docid) || !self.selectors.contains_key(&selectorid){
			log!("**[PawnScraper] Invalid html instances passed docid {:?},selectorid {:?}",docid,selectorid);
			Ok(-1)
		}else{
			let html = &self.html_instance.get(&docid).unwrap();
			let selector = &self.selectors.get(&selectorid).unwrap();
			let nth_element = html.select(selector).nth(idx);
			
			if nth_element == None{
				Ok(0)
			}else{
				let element_name = nth_element.unwrap().value().name();
				let name_encoded = samp_sdk::cp1251::encode(element_name).unwrap();
				set_string!(name_encoded,string,size);
				Ok(1)
			}
		}
	}
	
	fn get_nth_element_attr_value(&mut self,_:&AMX,docid:usize, selectorid:usize,idx:usize,attr:String,string:&mut Cell,size:usize) -> AmxResult<Cell>{
		if !self.html_instance.contains_key(&docid) || !self.selectors.contains_key(&selectorid){
			log!("**[PawnScraper] Invalid html instances passed docid {:?},selectorid {:?}",docid,selectorid);
			Ok(-1)
		}else{
			let html = &self.html_instance.get(&docid).unwrap();
			let selector = &self.selectors.get(&selectorid).unwrap();
			let nth_element = html.select(selector).nth(idx);
			if nth_element == None{
				Ok(0)
			}else{
				let attr_value = nth_element.unwrap().value().attr(&attr);
				if attr_value == None{
					Ok(-2)
				}else{
					let attr_encoded = samp_sdk::cp1251::encode(attr_value.unwrap()).unwrap();
					set_string!(attr_encoded,string,size);
					Ok(1)
				}
			}
		}
	}

	fn http_request(&mut self,_:&AMX,url:String) -> AmxResult<Cell>{
		match Request::new(&url){
			Ok(mut http) =>{
				match http.get().send(){
					Ok(res) => {
						let body = res.text();
						self.response_cache.insert(self.response_context_id,body);
						self.response_context_id += 1;
						Ok(self.response_context_id as Cell -1)
					}
					Err(err) =>{
						log!("**[PawnScraper] Http error {:?}",err);
						Ok(-1)
					}
				}
			}
			Err(err) =>{
				log!("**[PawnScraper] Url parse error {:?}",err);
				Ok(-1)
			}
		}
	}

	fn http_request_threaded(&mut self,_:&AMX,playerid:usize,callback:String,url:String) -> AmxResult<Cell>{
		self.request_send.as_ref().unwrap().send((playerid, callback, url)).unwrap();
		Ok(1)
	}


	fn delete_response_cache(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>{
		if self.response_cache.remove(&id) == None{
			log!("**[PawnScraper] Warning trying to remove invalid response id {:?}",id);
			Ok(0)
		}else{
			//log!("**[PawnScraper] Removed response_data {:?}",id);
			Ok(1)
		}
	}

	fn delete_html_instance(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>{
		if self.html_instance.remove(&id) == None{
			log!("**[PawnScraper] Warning trying to remove invalid html id {:?}",id);
			Ok(0)
		}else{
			//log!("**[PawnScraper] Removed html_instance {:?}",id);
			Ok(1)
		}
	}

	fn delete_selector_instance(&mut self,_:&AMX,id:usize) -> AmxResult<Cell>{
		if self.selectors.remove(&id) == None{
			log!("**[PawnScraper] Warning trying to remove invalid selector id {:?}",id);
			Ok(0)
		}else{
			//log!("**[PawnScraper] Removed selector_instance {:?}",id);
			Ok(1)
		}
	}	
}


