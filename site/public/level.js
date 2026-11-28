// Tell the CSS that scripts run, before first paint, so the phone menu starts closed
// instead of showing open and then collapsing (a big layout shift on phones).
document.documentElement.dataset.nav = 'js';

// Apply the saved level before first paint so the page doesn't flash beginner text.
try {
  var level = localStorage.getItem('cubesat.level');
  if (level === 'beginner' || level === 'intermediate' || level === 'professional') document.documentElement.dataset.level = level;
} catch (e) {}
