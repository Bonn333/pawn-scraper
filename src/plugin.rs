use internals::listen_for_http_calls;
use samp_sdk::amx::{AmxResult, AMX};
use samp_sdk::consts::*;
use samp_sdk::types::Cell;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

define_native!(parse_document, document: String);
define_native!(parse_document_by_response, id: usize);
define_native!(parse_selector, string: String);
define_native!(http_request, url: String, headerid: usize);
define_native!(delete_response_cache, id: usize);
define_native!(delete_html_instance, id: usize);
define_native!(delete_selector_instance, id: usize);
define_native!(delete_header_instance, id: usize);
define_native!(create_header as raw);

define_native!(
    http_request_threaded,
    playerid: usize,
    callback: String,
    url: String,
    headerid: usize
);

define_native!(
    get_nth_element_name,
    docid:usize,
    selectorid:usize,
    idx:usize,
    string:ref Cell,
    size:usize
);

define_native!(
    get_nth_element_text,
    docid:usize,
    selectorid:usize,
    idx:usize,
    string:ref Cell,
    size:usize
);

define_native!(
    get_nth_element_attr_value,
    docid:usize,
    selectorid:usize,
    idx:usize,
    attr:String,
    string:ref Cell,
    size:usize
);

pub struct PawnScraper {
    plugin_version: i32,
    pub html_instance: HashMap<usize, Html>,
    pub selectors: HashMap<usize, Selector>,
    pub response_cache: HashMap<usize, String>,
    pub header_instance: HashMap<usize, HashMap<String, String>>,
    pub html_context_id: usize,
    pub selector_context_id: usize,
    pub response_context_id: usize,
    pub header_context_id: usize,
    pub http_request_start_sender:
        Option<Sender<(usize, String, String, Option<HashMap<String, String>>)>>,
    pub http_request_complete_receiver: Option<Receiver<(usize, String, String, bool)>>,
    pub amx_list: Vec<usize>,
}

impl PawnScraper {
    pub fn load(&mut self) -> bool {
        listen_for_http_calls(self);

        log!(
            "
   ###############################################################
   #                      PawnScraper                            #
   #                        V0.2.0 Loaded!!                      #
   #   Found any bugs? Report it here:                           #
   #       https://github.com/Sreyas-Sreelal/pawn-scraper/issues #
   #                                                             #
   ###############################################################
			"
        );
        return true;
    }

    pub fn unload(&self) {
        log!("PawnScraper V0.2.0 Unloaded!!");
    }

    pub fn amx_load(&mut self, amx: &mut AMX) -> Cell {
        //log!("amx is {:?}",amx.amx);
        self.amx_list.push(amx.amx as usize);
        let natives = natives! {
            "ParseHtmlDocument" => parse_document,
            "ResponseParseHtml" => parse_document_by_response,
            "ParseSelector" => parse_selector,
            "HttpGet" => http_request,
            "HttpGetThreaded" => http_request_threaded,
            "GetNthElementName" => get_nth_element_name,
            "GetNthElementText" => get_nth_element_text,
            "GetNthElementAttrVal" => get_nth_element_attr_value,
            "DeleteHtml" => delete_html_instance,
            "DeleteSelector" => delete_selector_instance,
            "DeleteResponse" => delete_response_cache,
            "DeleteHeader" => delete_header_instance,
            "CreateHeader" => create_header
        };

        match amx.register(&natives) {
            Ok(_) => log!("**[PawnScraper] Natives are successfully loaded"),
            Err(err) => log!(
                "**[PawnScraper] There is an error loading natives {:?}",
                err
            ),
        }

        let get_version: AmxResult<&mut i32> = amx.find_pubvar("_pawnscraper_version");

        match get_version{
			Ok(version) =>{
				if *version != self.plugin_version{
					log!("**[PawnScraper] Warning plugin and include version doesnot match : Include {:?} Plugin {:?}",version,self.plugin_version);
				}
			},
			Err(err)=>{
				log!("**[PawnScraper] Failed to retrive include version Reasone:{:?}\n You might want to update include ", err)
			}
		}
        AMX_ERR_NONE
    }

    pub fn amx_unload(&mut self, amx: &mut AMX) -> Cell {
        let raw = amx.amx as usize;
        let index = self
            .amx_list
            .iter()
            .position(|x| *x == raw)
            .unwrap()
            .clone();
        self.amx_list.remove(index);
        AMX_ERR_NONE
    }

    pub fn process_tick(&mut self) {
        for (playerid, callback, body, success) in self
            .http_request_complete_receiver
            .as_ref()
            .unwrap()
            .try_iter()
        {
            let body = body.as_str();
            for amx in &self.amx_list {
                let amx = AMX::new(*amx as *mut _);
                let mut responseid = -1;
                let mut executed: bool;

                if success {
                    self.response_cache
                        .insert(self.response_context_id, String::from(body));
                    self.response_context_id += 1;
                    responseid = self.response_context_id as Cell - 1;
                }

                match exec_public_with_name!(amx,callback;playerid,responseid) {
                    Ok(_) => {
                        executed = true;
                    }
                    Err(_err) => {
                        continue;
                    }
                }

                if !executed {
                    log!("**[PawnScraper] Error executing callback {:?}", callback);
                }
            }
        }
    }
}

impl Default for PawnScraper {
    fn default() -> Self {
        PawnScraper {
            plugin_version: 20,
            html_instance: HashMap::new(),
            selectors: HashMap::new(),
            response_cache: HashMap::new(),
            header_instance: HashMap::new(),
            html_context_id: 0,
            selector_context_id: 0,
            response_context_id: 0,
            header_context_id: 0,
            http_request_start_sender: None,
            http_request_complete_receiver: None,
            amx_list: Vec::new(),
        }
    }
}
