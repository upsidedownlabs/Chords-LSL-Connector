import React from 'react';

type OSIconType = 'windows' | 'linux' | 'macos';

interface OSIconProps {
  os: OSIconType;
  size?: number;
  className?: string;
}

const OSIcon: React.FC<OSIconProps> = ({ os, size = 24, className }) => {
  const getIcon = () => {
    switch (os) {
      case 'windows':
        return (
          <svg
            width={size}
            height={size}
            viewBox="0 0 24 24"
            fill="currentColor"
            className={className}
            role="img"
            aria-label="Windows icon"
          >
            <path d="M3 12V6.5L10.5 5.5V12H3ZM3 17.5V12H10.5V18.5L3 17.5ZM11 5.5L21 4V12H11V5.5ZM21 12V20L11 18.5V12H21Z" />
          </svg>
        );

      case 'linux':
        return (
          <svg
            width={size}
            height={size}
            viewBox="0 0 32 32"
            fill="currentColor"
            className={className}
            role="img"
            aria-label="Linux icon"
            xmlns="http://www.w3.org/2000/svg"
          >
            {/* Optional: Replace with optimized or simplified Linux logo SVG */}
            <circle cx="16" cy="16" r="14" fill="black" />
            <text x="16" y="21" textAnchor="middle" fontSize="14" fill="white" fontFamily="Arial">
              🐧
            </text>
          </svg>
        );

      case 'macos':
        return (
          <svg
            width={size}
            height={size}
            viewBox="0 0 24 24"
            fill="currentColor"
            className={className}
            role="img"
            aria-label="macOS icon"
          >
            <path d="M18.71 19.5c-.83 1.24-1.71 2.45-3.05 2.47-1.34.03-1.77-.79-3.29-.79-1.53 0-2 .77-3.27.82-1.31.05-2.3-1.32-3.14-2.53C4.25 17 2.94 12.45 4.7 9.39 5.57 7.87 7.13 6.91 8.82 6.88c1.28-.02 2.5.87 3.29.87.78 0 2.26-1.07 3.81-.91.65.03 2.47.26 3.64 1.98-.09.06-2.17 1.28-2.15 3.81.03 3.02 2.65 4.03 2.68 4.04-.03.07-.42 1.44-1.38 2.83M13 3.5c.73-.83 1.94-1.46 2.94-1.5.13 1.17-.34 2.35-1.04 3.19-.69.85-1.83 1.51-2.95 1.42-.15-1.15.41-2.35 1.05-3.11z" />
          </svg>
        );

      default:
        return (
          <span className={className} aria-label="Unknown OS icon" role="img">
            ❓
          </span>
        );
    }
  };

  return getIcon();
};

export default OSIcon;
