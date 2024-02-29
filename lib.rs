//imporsts
use ic_cdk::api::management_canister::http_request::{
    http_request,CanisterHttpRequestArgument,HttpMethod};

use candid::{CandidType, Decode, Deserialize, Encode};
use ic_stable_structures::memory_manager::{
    MemoryId,MemoryManager,VirtualMemory};

use ic_stable_structures::{
    Boundedstorable,DefaultMemoryImpl,StableBTreeMap,Storable};

use std::borrow::Cow;
use std::str::EncodeUtf16;
use std::{barrow::Cow,cell::RefCell};

#[derive(CandidType, Deserialize, Clone)]
//struct oluşturma
struct Participant{
    address:String,
}
//event belirleme
#[derive(CandidType, Deserialize, Clone)]
struct  Event{
    name:String,
    date: String,
    #[serde(default)] //vektörün içini boşaltmak.
    participant: Vec<Participant>, //vektör
}

#[derive(CandidType,Deserialize)]
enum EventError {
    NoSuchEvent,
    JoinError,
    CancelJoinError,
    GetEventError,
    AlreadyJoined,
    AlreadyExist,
}
//implematation
impl Storable for Event {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(),Self).unwrap()
    }
}
type Memory=VirtualMemory<DefaultMemoryImpl>;
const MAX_VALUE_SIZE: u32 =100;

//implemantation  BoundedStorable for Event
impl Boundedstorable for Event{
    const MAX_SIZE:U32 =MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool =false;
}

//new memoryıd->thread_local!
thread_local! {
    static MEMORY_MENAGER: RefCell<MemoryManager<DefaultMemoryImpl>>=
    RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    
    static EVENTS_MAP :RefCell<StableBTreeMap<u64,Event,Memory>> = 
    RefCell::new(StableBTreeMap::init(MEMORY_MENAGER.with(|m| m.borrow().get    (MemoryId::new(1))), //farklı bir memoryId
        )
    ); 

}

// bir etkinlik yaratıp,depola

#[ic_cdk::update]
fn create_event(name: String,date:String)->Result<(),EventError> {
    EVENTS_MAP.with(|events_map_ref|{
        let mut events_map = events_map_ref.barrow_mut();

        //böyle bir etkinlik ismi var mı yok mu 

        for (_, event) in events_map.iter() {
            if event.name == name && event.date == date {
                return Err(EventError::AlreadyExists);
            }
        }

    //eğer bir etkinlik yoksa ,yeni bir tane oluştur
        let new_event = Event{
            name,
            date,
            participant:Vec::new(),
        };

        let new_event_id = events_map.len();
        events_map.insert(new_event_id,new_event);
        
        Ok(())
    
    })
}
    

#[ic_cdk::update]
fn join_event(event_id: u64, participant_address: String) -> Result<(), EventError> {
    EVENTS_MAP.with(|events_map_ref| {
        let mut events_map = events_map_ref.borrow_mut();
        // Retrieve the event, clone it, and then modify it
        if let Some(mut event) = events_map.get(&event_id) {
            if event.participants.iter().any(|p| p.address == participant_address) {
                return Err(EventError::AlreadyJoined);
            }

            let new_participant = Participant {address: participant_address};
            event.participants.push(new_participant);
            // Insert the modified event back into the map
            events_map.insert(event_id, event);
            Ok(())
        } else {
            Err(EventError::NoSuchEvent)
        }
    })
}

//katılımcının katılmayı düşündüğü etkinliğe katılmaması.

#[ic_cdk::update]
fn cancel_join_event(event_id: u64, participant_address: String) -> Result<(), EventError> {
    EVENTS_MAP.with(|events_map_ref| {
        let mut events_map = events_map_ref.borrow_mut();

        match events_map.get_mut(&event_id) {
            Some(event) => {
                match event.participants.iter().position(|p| p.address == participant_address) {
                    Some(index) => {
                        event.participants.remove(index);
                        events_map.insert(event_id, event.clone()); 
                    },
                    None => Err(EventError::CancelJoinError) // katılmadı
                }
            },
            None => Err(EventError::NoSuchEvent) // Event yok 
        }
    })
}
