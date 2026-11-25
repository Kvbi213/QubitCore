<script lang="ts">
  let { size = 24, class: className = '' }: { size?: number | string; class?: string } = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 128 128"
  width={size}
  height={size}
  fill="none"
  class={className}
>
  <defs>
    <!-- Rozproszona poświata Plasma Glow -->
    <radialGradient id="qb-aura" cx="47%" cy="47%" r="50%">
      <stop offset="0%" stop-color="#00F0FF" stop-opacity="0.6" />
      <stop offset="48%" stop-color="#8B5CF6" stop-opacity="0.3" />
      <stop offset="100%" stop-color="#050507" stop-opacity="0" />
    </radialGradient>

    <!-- Wewnętrzna poświata pod znakiem nieskończoności -->
    <radialGradient id="qb-center-glow" cx="50%" cy="50%" r="50%">
      <stop offset="0%" stop-color="#00F0FF" stop-opacity="0.45" />
      <stop offset="65%" stop-color="#A855F7" stop-opacity="0.2" />
      <stop offset="100%" stop-color="#050507" stop-opacity="0" />
    </radialGradient>

    <!-- Gradient głównego pierścienia Q -->
    <linearGradient id="qb-ring" x1="22" y1="22" x2="98" y2="98" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#00F0FF" />
      <stop offset="52%" stop-color="#8B5CF6" />
      <stop offset="100%" stop-color="#EC4899" />
    </linearGradient>

    <!-- Gradient przedniej orbity -->
    <linearGradient id="qb-orbit" x1="8" y1="60" x2="112" y2="60" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#00F0FF" stop-opacity="0.25" />
      <stop offset="50%" stop-color="#E0F2FE" stop-opacity="0.95" />
      <stop offset="100%" stop-color="#A855F7" stop-opacity="0.35" />
    </linearGradient>

    <!-- Gradient ogonka Q -->
    <linearGradient id="qb-tail" x1="84" y1="84" x2="108" y2="108" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#8B5CF6" />
      <stop offset="100%" stop-color="#00F0FF" />
    </linearGradient>
  </defs>

  <!-- Tło poświaty -->
  <circle cx="60" cy="60" r="54" fill="url(#qb-aura)" />

  <!-- Tylna część orbity 3D (za pierścieniem) -->
  <g transform="rotate(-32 60 60)">
    <path d="M 7 60 A 53 16 0 0 1 113 60" fill="none" stroke="#38BDF8" stroke-opacity="0.35" stroke-width="2.5" stroke-linecap="round" />
  </g>

  <!-- Pełny, zamknięty okrąg Q -->
  <circle cx="60" cy="60" r="37" fill="none" stroke="url(#qb-ring)" stroke-width="7.5" />

  <!-- Poświata za znakiem nieskończoności -->
  <circle cx="60" cy="60" r="24" fill="url(#qb-center-glow)" />

  <!-- Poświata samego znaku nieskończoności -->
  <path d="M 60 60 C 55 52, 44 52, 44 60 C 44 68, 55 68, 60 60 Z" fill="none" stroke="#00F0FF" stroke-opacity="0.5" stroke-width="6.5" stroke-linecap="round" stroke-linejoin="round" />

  <!-- Ostry, biały znak nieskończoności w centrum -->
  <path d="M 60 60 C 55 52, 44 52, 44 60 C 44 68, 55 68, 60 60 Z" fill="none" stroke="#FFFFFF" stroke-width="3.2" stroke-linecap="round" stroke-linejoin="round" />

  <!-- Przednia część orbity 3D -->
  <g transform="rotate(-32 60 60)">
    <path d="M 7 60 A 53 16 0 0 0 113 60" fill="none" stroke="url(#qb-orbit)" stroke-width="3" stroke-linecap="round" />
  </g>

  <!-- Ogonek litery Q -->
  <path d="M 85 85 L 108 108" fill="none" stroke="url(#qb-tail)" stroke-width="8" stroke-linecap="round" />
</svg>
