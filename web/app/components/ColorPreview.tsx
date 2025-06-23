import { ColorScale } from "@/lib/colors";
import { useMemo } from "react";

function generateGradient(scale: ColorScale): string {
  let stops: string[] = [];
  const step = 0.1;
  for (let i = 0; i <= 100; i += step) {
    const light = scale.light(i / 100).css();
    const dark = scale.dark(i / 100).css();
    stops.push(`light-dark(${light}, ${dark}) ${i}%`);
    // stops.push(`${props.scale((i - 0.0001) / 100).css()} ${i}%`);
  }
  const gradient = `linear-gradient(to right, ${stops.join(",")})`;
  return gradient;
}

export default function ColorPreview(props: { scale: ColorScale }) {
  const gradient = useMemo(() => {
    return generateGradient(props.scale);
  }, [props.scale]);

  const stops = [0.1, 0.35, 0.5, 0.65, 0.9];
  return (
    <div className="flex flex-col relative">
      <div className="h-[24px] w-full rounded" style={{ background: gradient }}>
        {stops.map((x) => (
          <div
            className="border-l-3 border-white dark:border-black w-0"
            key={x}
            style={{
              position: "absolute",
              left: `${x * 100}%`,
              //   transform: "translateX(-50%)",
            }}
          >
            &nbsp;
          </div>
        ))}
        <div>&nbsp;</div>
      </div>
      <div className="flex flex-row mb-1">
        <div>&nbsp;</div>
        <div
          style={{
            position: "absolute",
            left: "0%",
          }}
        >
          worst
        </div>
        {stops.map((x) => {
          return (
            <div
              key={x}
              className="text-center"
              style={{
                position: "absolute",
                left: `${x * 100}%`,
                transform: "translateX(-40%)",
              }}
            >
              {Math.round(x * 100)}%
            </div>
          );
        })}
        <div
          className="text-right"
          style={{
            position: "absolute",
            right: "0%",
          }}
        >
          best
        </div>
      </div>
    </div>
  );
}
