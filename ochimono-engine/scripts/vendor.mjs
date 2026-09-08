import { copyFile, mkdir, readFile } from 'node:fs/promises';

const root = new URL('../', import.meta.url);
const source = new URL('pkg-npm/', root);
const destination = new URL('../ochimono-ui/vendor/ochimono-engine/', root);
const manifest = JSON.parse(await readFile(new URL('package.json', source), 'utf8'));
await mkdir(destination, { recursive: true });
for (const file of [...manifest.files, 'package.json']) {
    await copyFile(new URL(file, source), new URL(file, destination));
}
await copyFile(new URL('README.md', root), new URL('README.md', destination));
await copyFile(new URL('../LICENSE', root), new URL('LICENSE', destination));
await copyFile(
    new URL('tests/fixtures/replay.json', root),
    new URL('../ochimono-ui/src/lib/game/fixtures/replay.json', root)
);
