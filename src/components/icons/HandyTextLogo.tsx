import React from "react";

// ХЭНДИ wordmark — static outlines, no text elements and no font dependency, so
// the logotype renders identically on Windows, macOS and Linux.
const HandyTextLogo = ({
  width,
  height,
  className,
}: {
  width?: number;
  height?: number;
  className?: string;
}) => {
  return (
    <svg
      width={width}
      height={height}
      className={className}
      viewBox="-1.4 -144.3 742.8 184.5"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Generated from segoeuib.ttf (Segoe UI Bold) by
          scripts/generate-wordmark.py: text 'ХЭНДИ', cap height 140,
          tracking 0.06em. The font is not shipped — only these outlines are. */}
      <path
        d="M130.3 0H92.9L68.8 -45.6Q67.5 -48.1 66 -55H65.6Q64.9 -51.7 62.4 -45.2L38.2 0H0.6L45.2 -70L4.4 -140H42.8L62.8 -98Q65.1 -93 67 -86.2H67.4Q68.5 -90.3 71.8 -98.4L94 -140H129.2L87.2 -70.6Z M148 -5V-32.4Q163.3 -23.4 182.6 -23.4Q217.8 -23.4 222.4 -57H168.4V-82H222.6Q216.5 -116.6 182.8 -116.6Q163.5 -116.6 148 -108.4V-136.8Q164.3 -142.3 186 -142.3Q219.3 -142.3 238 -123.2Q256.8 -104 256.8 -69.6Q256.8 -37.1 238.1 -17.3Q219.5 2.4 187.2 2.4Q163.4 2.4 148 -5Z M409.7 0H378V-57H320V0H288.5V-140H320V-84.2H378V-140H409.7Z M581.8 38.2H554.1V0H465.2V38.2H437.9V-25.8H450.9Q478 -74.9 480.8 -140H561.9V-25.8H581.8ZM530.2 -25.8V-113.7H506.4Q505.3 -94.3 499 -70Q492.6 -45.7 483.3 -25.8Z M739.4 0H709.6V-76.9Q709.6 -90.2 710.6 -99.6H710Q707.9 -95.6 702.9 -88L645.1 0H613.4V-140H643.2V-62.6Q643.2 -48.2 642.4 -43.3H642.8Q643.3 -44.5 650 -54.8L705.6 -140H739.4Z"
        className="logo-primary"
      />
    </svg>
  );
};

export default HandyTextLogo;
