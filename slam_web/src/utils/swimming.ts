import {
  type Sport,
  SportType,
  type Swimming,
  type Track,
  getSportType,
} from '../services/sport';

export const DEFAULT_LANE_LENGTH_METER = 25;

export function getSwimmingLaneLength(sport: Sport): number {
  if (getSportType(sport.type) !== SportType.Swimming) return 0;
  const configured = (sport.extra as Swimming | undefined)?.lane_length_meter;
  if (configured && configured > 0) return configured;
  const firstTrackLength = sport.tracks.find(
    track => track.distance_meter > 0,
  )?.distance_meter;
  return firstTrackLength || DEFAULT_LANE_LENGTH_METER;
}

export function withSwimmingLaneLength(
  sport: Sport,
  laneLength: number,
  sourceTracks: Track[] = sport.tracks,
): Sport {
  const normalizedLength = Math.max(1, Math.trunc(laneLength));
  const currentExtra = (sport.extra as Swimming | undefined) ?? {
    main_stroke: 'unknown',
    stroke_avg: 0,
    swolf_avg: 0,
  };
  const tracks = sourceTracks.map(track => ({
    ...track,
    distance_meter: normalizedLength,
  }));
  return {
    ...sport,
    extra: {
      ...currentExtra,
      lane_length_meter: normalizedLength,
    },
    tracks,
    distance_meter: normalizedLength * tracks.length,
  };
}

export function withSwimmingTracks(sport: Sport, tracks: Track[]): Sport {
  if (getSportType(sport.type) !== SportType.Swimming) {
    return { ...sport, tracks };
  }
  return withSwimmingLaneLength(sport, getSwimmingLaneLength(sport), tracks);
}

export function initializeSwimmingSport(sport: Sport): Sport {
  if (getSportType(sport.type) !== SportType.Swimming) return sport;
  const laneLength = getSwimmingLaneLength(sport);
  const currentExtra = (sport.extra as Swimming | undefined) ?? {
    main_stroke: 'unknown',
    stroke_avg: 0,
    swolf_avg: 0,
  };
  return {
    ...sport,
    extra: { ...currentExtra, lane_length_meter: laneLength },
  };
}
