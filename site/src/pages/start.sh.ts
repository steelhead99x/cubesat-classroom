import installer from '../../../docs/start.sh?raw';

export function GET() {
  return new Response(installer, {
    headers: { 'content-type': 'text/plain; charset=utf-8' },
  });
}
