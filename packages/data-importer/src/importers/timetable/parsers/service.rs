use crate::importers::timetable::parsers::service::station_event::StationEventType;
use attribute::Attribute;
use identification::ServiceIdentification;
use number::ServiceNumber;
use platform_info::PlatformInfo;
use station_event::StationEvent;
use std::iter::Filter;
use std::ops::Index;
use std::slice::Iter;
use transport_mode::TransportMode;
use validity::Validity;

pub mod attribute;
pub mod identification;
pub mod number;
pub mod platform_info;
pub mod station_event;
pub mod transport_mode;
pub mod validity;

#[derive(Debug, PartialEq, Clone)]
pub struct Service {
    pub identification: ServiceIdentification,
    pub service_number: Vec<ServiceNumber>,
    pub validity: Validity,
    pub transport_mode: TransportMode,
    pub attributes: Vec<Attribute>,
    pub station_events: Vec<(StationEvent, Option<PlatformInfo>)>,
}

impl Service {
    pub fn stop_at(&self, stop_index: u32) -> Option<(StationEvent, PlatformInfo)> {
        let (event, platform_info) = self
            .station_events
            .iter()
            .filter(|(event, _)| event.stop_type != StationEventType::Passage)
            .nth((stop_index - 1) as usize)?;

        if platform_info.is_none() {
            return None;
        }

        Some((event.clone(), platform_info.clone().unwrap()))
    }

    pub fn split_legs(&self) -> Vec<ServiceLeg> {
        if self.service_number.len() == 1 {
            return vec![ServiceLeg {
                service_number: self.service_number[0].clone(),
                validity: self.validity.clone(),
                transport_mode: self.transport_mode.clone(),
                attributes: self.attributes.clone(),
                station_events: self.station_events.clone(),
            }];
        }

        if self.service_number.len() == 2 {
            let split_stop = self
                .station_events
                .iter()
                .filter(|(e, _)| e.stop_type != StationEventType::Passage)
                .collect::<Vec<_>>()[self.service_number[0].last_stop as usize - 1];

            let split_index = self
                .station_events
                .iter()
                .position(|(e, _)| e.station == *split_stop.0.station)
                .unwrap();

            let split_event = self.station_events[split_index].clone();
            let new_arrival = (split_event.clone().0.into_arrival(), split_event.clone().1);
            let new_departure = (
                split_event.clone().0.into_departure(),
                split_event.clone().1,
            );

            let mut new_events_leg_1 = self.station_events[0..split_index].to_vec();
            new_events_leg_1.push(new_arrival);

            let mut new_events_leg_2 = self.station_events[split_index + 1..].to_vec();
            new_events_leg_2.insert(0, new_departure);

            return vec![
                ServiceLeg {
                    service_number: self.service_number[0].clone(),
                    validity: self.validity.clone(),
                    transport_mode: self.transport_mode.clone(),
                    attributes: self.attributes.clone(),
                    station_events: new_events_leg_1,
                },
                ServiceLeg {
                    service_number: self.service_number[1].clone(),
                    validity: self.validity.clone(),
                    transport_mode: self.transport_mode.clone(),
                    attributes: self.attributes.clone(),
                    station_events: new_events_leg_2,
                },
            ];
        }

        panic!();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceLeg {
    pub service_number: ServiceNumber,
    pub validity: Validity,
    pub transport_mode: TransportMode,
    pub attributes: Vec<Attribute>,
    pub station_events: Vec<(StationEvent, Option<PlatformInfo>)>,
}

impl ServiceLeg {
    pub fn stops(&self) -> impl Iterator<Item = &(StationEvent, Option<PlatformInfo>)> {
        self.station_events
            .iter()
            .filter(|(e, _)| e.stop_type != StationEventType::Passage)
    }

    pub fn num_stops(&self) -> u32 {
        self.stops().count() as u32
    }

    pub fn stop_number(&self, event: &StationEvent) -> Option<u32> {
        if event.stop_type == StationEventType::Passage {
            return None;
        }

        self.stops()
            .position(|(e, _)| e == event)
            .map(|pos| pos as u32)
    }
}
