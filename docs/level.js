// Apply the saved level before first paint so the page doesn't flash beginner text.
try {
  var level = localStorage.getItem('cubesat.level');
  if (level === 'beginner' || level === 'intermediate' || level === 'professional') document.documentElement.dataset.level = level;
} catch (e) {}
