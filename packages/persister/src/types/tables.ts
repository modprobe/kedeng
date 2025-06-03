declare module "knex/types/tables" {
  interface Service {
    id: string;
    train_number: string;
    timetable_year: string;
    type: string;
    provider: string;
  }

  interface Journey {
    id: string;
    service_id: string;
    running_on: string;
  }

  interface JourneyEvent {
    id: string;
    journey_id: string;
    station: string;
    event_type_planned: string;
    event_type_actual: string;
    stop_order: number;

    arrival_time_planned: string;
    arrival_time_actual: string;
    arrival_platform_planned: string;
    arrival_platform_actual: string;

    departure_time_planned: string;
    departure_time_actual: string;
    departure_platform_planned: string;
    departure_platform_actual: string;

    status: number;
    is_cancelled: boolean;
    attributes: string[];
  }

  interface Tables {
    service: Service;
    journey: Journey;
    journey_event: JourneyEvent;
  }
}
