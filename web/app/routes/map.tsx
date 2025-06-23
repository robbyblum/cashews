import { getLocations, MapLocation } from "@/lib/data";
import { createFileRoute } from "@tanstack/react-router";
import { Circle, MapContainer, Marker, Popup, TileLayer } from "react-leaflet";

import mapCss from "../styles/map.css?url";

import { DivIcon, Icon, LatLng, Point } from "leaflet";
import { useMemo } from "react";

// thanks https://stackoverflow.com/questions/2450954/how-to-randomize-shuffle-a-javascript-array
function shuffle(array) {
  let currentIndex = array.length;

  // While there remain elements to shuffle...
  while (currentIndex != 0) {
    // Pick a remaining element...
    let randomIndex = Math.floor(Math.random() * currentIndex);
    currentIndex--;

    // And swap it with the current element.
    [array[currentIndex], array[randomIndex]] = [
      array[randomIndex],
      array[currentIndex],
    ];
  }
}

export const Route = createFileRoute("/map")({
  component: RouteComponent,
  loader: async () => {
    const locations = await getLocations();

    // prevent bias in team order and "randomize" visibility
    shuffle(locations);

    return { locations };
  },
  head: () => ({
    links: [
      {
        rel: "stylesheet",
        href: mapCss,
      },
    ],
  }),
  // leaflet doesn't support ssr
  ssr: false,
});

function RouteComponent() {
  const { locations } = Route.useLoaderData() as { locations: MapLocation[] };

  const latLongCounts = {};
  for (const team of locations) {
    if (team.location) {
      const latLong = [team.location.lat, team.location.long];
      const latLongStr = JSON.stringify(latLong);
      latLongCounts[latLongStr] = (latLongCounts[latLongStr] || 0) + 1;
    }
  }

  const JITTER_RADIUS = 500; // in meters

  const { rangeCircles, markers } = useMemo(() => {
    const rangeCircles: [number, number][] = [];
    for (const key of Object.keys(latLongCounts)) {
      if (latLongCounts[key] > 1) {
        rangeCircles.push(JSON.parse(key));
      }
    }

    const markers: [LatLng, MapLocation, Icon][] = [];
    for (const team of locations) {
      if (team.location) {
        const latLong = [team.location.lat, team.location.long];
        const latLongStr = JSON.stringify(latLong);

        let jitterLatlong;

        if (latLongCounts[latLongStr] > 1) {
          // slight random jitter so you can always see overlapping teams
          const baseLatlong = new LatLng(team.location.lat, team.location.long);
          const bounds = baseLatlong.toBounds(JITTER_RADIUS * 2);
          const latRange = bounds.getNorth() - bounds.getSouth();
          const longRange = bounds.getEast() - bounds.getWest();

          do {
            jitterLatlong = new LatLng(
              bounds.getSouth() + Math.random() * latRange,
              bounds.getWest() + Math.random() * longRange
            );
          } while (jitterLatlong.distanceTo(baseLatlong) > JITTER_RADIUS);
        } else {
          jitterLatlong = latLong;
        }

        const icon = new DivIcon({
          className: "emoji",
          html: `<span>` + team.team.emoji + `</span>`,
          iconSize: new Point(40, 40),
          popupAnchor: new Point(0, -12),
        });
        markers.push([jitterLatlong, team, icon as Icon]);
      }
    }
    return { rangeCircles, markers };
  }, [locations]);

  return (
    <MapContainer
      style={{ width: "100%", height: "100%" }}
      center={[51.505, -0.09]}
      zoom={3}
      className="w-full h-full cashews-map"
    >
      <TileLayer
        attribution={
          '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors &copy; <a href="https://carto.com/attributions">CARTO</a>'
        }
        subdomains="abcd"
        url="https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png"
      />

      {markers.map(([m, team, icon]) => {
        return (
          <Marker key={team.team.team_id} position={m} icon={icon}>
            <Popup>
              <a
                className="flex flex-col items-center text-center hover:underline"
                href={`https://mmolb.com/team/${team.team.team_id}`}
              >
                <strong className="">
                  {team.team.emoji} {team.team.location} {team.team.name}
                </strong>
                <br />
                {team.team.full_location}
              </a>
            </Popup>
          </Marker>
        );
      })}

      {rangeCircles.map((pos) => {
        return (
          <Circle
            key={JSON.stringify(pos)}
            center={pos}
            radius={JITTER_RADIUS}
            fillOpacity={0.1}
            opacity={0.5}
            dashArray="10 10"
          />
        );
      })}
    </MapContainer>
  );
}
