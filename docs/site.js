const button = document.querySelector('#copy');
const command = document.querySelector('#command');
const status = document.querySelector('#copy-status');

if (navigator.clipboard?.writeText) {
  button.hidden = false;
  button.addEventListener('click', async () => {
    try {
      await navigator.clipboard.writeText(command.textContent.trim());
      status.textContent = 'Copied. Paste the command into your terminal and press Enter.';
    } catch {
      status.textContent = 'Select and copy the command above.';
    }
  });
}
