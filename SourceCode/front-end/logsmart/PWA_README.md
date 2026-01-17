# LogSmart PWA Implementation

LogSmart has been transformed into a Progressive Web App (PWA), enabling mobile users to install it on their devices and use it offline like a native application.

## PWA Features

### 📱 Installability

- **Desktop**: Install prompt appears automatically on supported browsers
- **iOS**: Manual installation via Safari Share button → "Add to Home Screen"
- **Android**: Automatic install prompt with custom banner

### 🔌 Offline Support

- Cached static assets (JS, CSS, images, fonts)
- Offline fallback page when network is unavailable
- Smart caching strategies:
  - **Static resources**: Cache-first
  - **API calls**: Network-first with 5-minute cache
  - **Google Fonts**: Cache-first with 1-year expiration

### 🎨 Native App Experience

- Standalone display mode (no browser UI)
- Custom theme color (#0b172a)
- Splash screen support
- Apple touch icons
- Maskable icons for adaptive display

## Files Added/Modified

### New Files

- `src/lib/components/pwa_install_prompt.svelte` - Install banner component
- `src/pwa.d.ts` - TypeScript definitions for PWA virtual modules
- `static/offline.html` - Offline fallback page
- `static/pwa-icon-192.png` - PWA icon (192x192)
- `static/pwa-icon-512.png` - PWA icon (512x512)
- `static/pwa-icon-512-maskable.png` - Maskable icon for adaptive display

### Modified Files

- `vite.config.ts` - Added @vite-pwa/sveltekit plugin configuration
- `src/app.html` - Added PWA meta tags and manifest link
- `src/routes/+layout.svelte` - Integrated install prompt component
- `package.json` - Added @vite-pwa/sveltekit dependency

## Testing the PWA

### Local Development

```bash
bun run dev
```

The PWA is enabled in development mode for testing.

### Production Build

```bash
bun run build
bun run preview
```

### Testing Installation

#### Desktop (Chrome/Edge)

1. Open the app in browser
2. Look for install icon in address bar
3. Or use the install banner that appears

#### iOS (Safari)

1. Open app in Safari
2. Tap Share button
3. Scroll and tap "Add to Home Screen"
4. Confirm installation

#### Android (Chrome)

1. Open app in Chrome
2. Install banner appears automatically
3. Or use browser menu → "Install app"

### Testing Offline Functionality

1. Install the PWA
2. Open DevTools → Application → Service Workers
3. Check "Offline" mode
4. Navigate through the app
5. Previously visited pages should work
6. New pages show offline fallback

## Caching Strategies

### Static Assets (Cache-First)

- JavaScript bundles
- CSS files
- Images and icons
- Web fonts

### API Endpoints (Network-First)

- `/api/*` routes
- 10-second network timeout
- Falls back to 5-minute cache
- Maximum 50 cached entries

### External Resources (Cache-First)

- Google Fonts (1-year expiration)
- Font files (1-year expiration)

## Manifest Configuration

The web app manifest is auto-generated and includes:

- **Name**: LogSmart
- **Short Name**: LogSmart
- **Theme Color**: #0b172a
- **Background Color**: #0b172a
- **Display**: standalone
- **Start URL**: /
- **Icons**: 192x192, 512x512 (regular + maskable)

## Service Worker

Auto-generated using Workbox with:

- **Strategy**: generateSW
- **Precached files**: ~68 entries (~567 KB)
- **Runtime caching**: API, fonts, external resources
- **Offline fallback**: Custom offline page

## Browser Support

### Full PWA Support

- Chrome/Edge (Desktop & Mobile)
- Safari 16.4+ (iOS & macOS)
- Firefox (Desktop & Android)
- Samsung Internet

### Limited Support

- Safari < 16.4 (missing some features)
- Older browsers (no service worker)

## Deployment Notes

### Cloudflare Pages

The app uses `@sveltejs/adapter-cloudflare`. Service workers are automatically deployed with the app bundle.

### HTTPS Requirement

PWAs require HTTPS. Local development works on localhost, but production must use HTTPS.

## Troubleshooting

### Install prompt doesn't appear

- Check browser console for errors
- Ensure HTTPS is enabled (not required for localhost)
- Verify manifest is accessible at `/manifest.webmanifest`
- Check service worker registration in DevTools

### Offline mode not working

- Open DevTools → Application → Service Workers
- Verify service worker is activated
- Check cache storage for precached assets
- Ensure you've visited pages while online first

### Icons not displaying

- Verify icon files exist in `static/` directory
- Check manifest.webmanifest for correct paths
- Clear browser cache and reinstall

### iOS installation issues

- Use Safari browser (Chrome on iOS doesn't support PWA install)
- Ensure all meta tags are present in app.html
- Check for apple-touch-icon availability

## Updating the PWA

When you deploy updates:

1. Service worker detects new version
2. New assets are downloaded in background
3. User sees update notification (future enhancement)
4. User can refresh to get new version

## Future Enhancements

- [ ] Update notification with reload button
- [ ] Push notifications for log alerts
- [ ] Background sync for offline form submissions
- [ ] Enhanced offline capabilities for specific routes
- [ ] Install analytics tracking
- [ ] A/B testing for install prompts
