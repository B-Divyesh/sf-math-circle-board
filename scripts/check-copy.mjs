import {readFileSync} from 'node:fs';

const read = path => readFileSync(new URL(`../${path}`, import.meta.url), 'utf8');
const readme = read('README.md');
const source = read('frontend/src/main.ts');
const catalog = read('.factory/catalog-description.txt').trim();
const claims = JSON.parse(read('.factory/claims.json'));
const browserTests = read('tests/e2e.spec.ts');

const failures = [];
const requireText = (condition, message) => {
  if (!condition) failures.push(message);
};

requireText(source.includes('Data kept on the board'), 'The landing page needs the informative privacy heading.');
requireText(!source.includes('Keep only the record you need'), 'The slogan heading must not return.');
requireText(!source.includes('Original AI-assisted environmental art'), 'Public copy must not make the untested provenance claim.');
requireText(!readme.includes('manifest claim'), 'README must explain claim checks without manifest jargon.');
requireText(readme.includes('runs the named demo-isolation check on a fresh local server.'), 'README needs the plain demo-isolation explanation.');
requireText(!readme.includes('The browser suite covers desktop and 390 px layouts'), 'The long browser-suite sentence must stay split.');
requireText(catalog.length <= 120, `Catalog description is ${catalog.length} characters; maximum is 120.`);
requireText(/^(Record|Plan)\b/.test(catalog), 'Catalog description must start with a verb.');

const ids = claims.map(claim => claim.id);
requireText(new Set(ids).size === ids.length, 'Claim IDs must be unique.');
for (const claim of claims) {
  const tag = `@claim:${claim.id}`;
  const count = browserTests.split(tag).length - 1;
  requireText(count === 1, `${tag} must appear in exactly one browser test; found ${count}.`);
  requireText(claim.test.includes(tag), `${claim.id} command must select its matching tag.`);
}

if (failures.length) {
  console.error(failures.join('\n'));
  process.exit(1);
}

console.log(`Copy contract passed; ${claims.length} claims each map to one browser test.`);
