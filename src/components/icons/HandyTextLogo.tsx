import React from "react";

// ХЭНДИ wordmark — static outlines, no <text> and no font dependency, so the
// logotype renders identically on Windows, macOS and Linux.
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
      viewBox="0 -140 755.5 140"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      {/* Generated from segoeuib.ttf (Segoe UI Bold) by
          scripts/generate-wordmark.py: text 'ХЭНДИ', cap height 140,
          tracking 0.06em. The font is not shipped — only these outlines are. */}
      <path
        d="M12.7 0H9.1L6.7 -4.5Q6.6 -4.7 6.4 -5.4H6.4Q6.3 -5.1 6.1 -4.4L3.7 0H0.1L4.4 -6.8L0.4 -13.7H4.2L6.1 -9.6Q6.4 -9.1 6.5 -8.4H6.6Q6.7 -8.8 7 -9.6L9.2 -13.7H12.6L8.5 -6.9Z M14.5 -0.5V-3.2Q15.9 -2.3 17.8 -2.3Q21.3 -2.3 21.7 -5.6H16.4V-8H21.7Q21.1 -11.4 17.8 -11.4Q16 -11.4 14.5 -10.6V-13.4Q16 -13.9 18.2 -13.9Q21.4 -13.9 23.2 -12Q25.1 -10.2 25.1 -6.8Q25.1 -3.6 23.2 -1.7Q21.4 0.2 18.3 0.2Q16 0.2 14.5 -0.5Z M40 0H36.9V-5.6H31.2V0H28.2V-13.7H31.2V-8.2H36.9V-13.7H40Z M56.8 3.7H54.1V0H45.4V3.7H42.7V-2.5H44Q46.7 -7.3 46.9 -13.7H54.9V-2.5H56.8ZM51.8 -2.5V-11.1H49.4Q49.3 -9.2 48.7 -6.8Q48.1 -4.5 47.2 -2.5Z M72.2 0H69.3V-7.5Q69.3 -8.8 69.4 -9.7H69.3Q69.1 -9.3 68.6 -8.6L63 0H59.9V-13.7H62.8V-6.1Q62.8 -4.7 62.7 -4.2H62.8Q62.8 -4.3 63.5 -5.3L68.9 -13.7H72.2Z"
        className="logo-primary"
      />
    </svg>
  );
};

export default HandyTextLogo;
