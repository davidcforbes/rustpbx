const puppeteer = require('puppeteer');
const markdownIt = require('markdown-it');
const fs = require('fs');
const path = require('path');

const md = markdownIt({ html: true, linkify: true, typographer: true });

// Collect mermaid blocks and replace with placeholders
let mermaidBlocks = [];
const defaultFence = md.renderer.rules.fence.bind(md.renderer.rules);
md.renderer.rules.fence = (tokens, idx, options, env, self) => {
  const token = tokens[idx];
  if (token.info.trim() === 'mermaid') {
    const id = mermaidBlocks.length;
    mermaidBlocks.push(token.content);
    return `<div class="mermaid-placeholder" id="mermaid-${id}"></div>`;
  }
  return defaultFence(tokens, idx, options, env, self);
};

async function convertMdToPdf(inputFile, outputFile) {
  console.log(`Reading: ${inputFile}`);
  mermaidBlocks = [];
  const markdown = fs.readFileSync(inputFile, 'utf-8');
  const htmlBody = md.render(markdown);
  const title = path.basename(inputFile, '.md');
  console.log(`  Found ${mermaidBlocks.length} mermaid diagrams`);

  // Build mermaid rendering script that processes one diagram at a time
  const mermaidData = JSON.stringify(mermaidBlocks);

  const html = `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>${title}</title>
<style>
  body {
    font-family: 'Segoe UI', -apple-system, BlinkMacSystemFont, sans-serif;
    line-height: 1.6;
    color: #1a1a1a;
    max-width: 900px;
    margin: 0 auto;
    padding: 40px 50px;
    font-size: 11pt;
  }
  h1 { font-size: 22pt; color: #0d1117; border-bottom: 2px solid #0969da; padding-bottom: 8px; margin-top: 32px; }
  h2 { font-size: 16pt; color: #1a1a1a; border-bottom: 1px solid #d0d7de; padding-bottom: 6px; margin-top: 28px; }
  h3 { font-size: 13pt; color: #24292f; margin-top: 24px; }
  h4 { font-size: 11pt; color: #24292f; margin-top: 20px; }
  table { border-collapse: collapse; width: 100%; margin: 16px 0; font-size: 10pt; }
  th { background: #f0f4f8; font-weight: 600; text-align: left; padding: 8px 12px; border: 1px solid #d0d7de; }
  td { padding: 8px 12px; border: 1px solid #d0d7de; vertical-align: top; }
  tr:nth-child(even) { background: #f6f8fa; }
  code { background: #f0f4f8; padding: 2px 6px; border-radius: 3px; font-size: 10pt; font-family: 'Cascadia Code', Consolas, monospace; }
  pre { background: #f6f8fa; padding: 16px; border-radius: 6px; overflow-x: auto; border: 1px solid #d0d7de; }
  pre code { background: none; padding: 0; }
  blockquote { border-left: 4px solid #0969da; margin: 16px 0; padding: 8px 16px; background: #f0f7ff; color: #24292f; }
  a { color: #0969da; text-decoration: none; }
  strong { color: #0d1117; }
  em { color: #57606a; }
  hr { border: none; border-top: 1px solid #d0d7de; margin: 24px 0; }
  ul, ol { padding-left: 24px; }
  li { margin: 4px 0; }
  .mermaid-placeholder { text-align: center; margin: 20px 0; }
  .mermaid-placeholder svg { max-width: 100%; }
  .mermaid-error { background: #fff3cd; border: 1px solid #ffc107; padding: 10px; border-radius: 4px; color: #856404; font-size: 9pt; }

  @media print {
    body { padding: 0; }
    h1, h2 { page-break-after: avoid; }
    table, pre, .mermaid-placeholder { page-break-inside: avoid; }
  }
</style>
</head>
<body>
${htmlBody}
<script type="module">
  import mermaid from 'https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.esm.min.mjs';
  mermaid.initialize({ startOnLoad: false, theme: 'default', securityLevel: 'loose', maxTextSize: 500000 });

  const diagrams = ${mermaidData};
  let rendered = 0;
  let errors = 0;

  for (let i = 0; i < diagrams.length; i++) {
    const el = document.getElementById('mermaid-' + i);
    if (!el) continue;
    try {
      const { svg } = await mermaid.render('mermaid-svg-' + i, diagrams[i]);
      el.innerHTML = svg;
      rendered++;
    } catch (e) {
      el.innerHTML = '<div class="mermaid-error">Diagram ' + i + ' error: ' + e.message + '</div>';
      errors++;
      console.error('Diagram ' + i + ' failed:', e.message);
    }
  }
  console.log('MERMAID_DONE:' + rendered + ':' + errors + ':' + diagrams.length);
</script>
</body>
</html>`;

  console.log('Launching browser...');
  const browser = await puppeteer.launch({ headless: true });
  const page = await browser.newPage();

  // Capture browser console
  page.on('console', msg => {
    const text = msg.text();
    if (text.startsWith('MERMAID_DONE:')) {
      const [_, r, e, t] = text.split(':');
      console.log(`  Diagrams: ${t} total, ${r} rendered, ${e} errors`);
    } else if (msg.type() === 'error') {
      console.log(`  [ERROR] ${text.substring(0, 200)}`);
    }
  });
  page.on('pageerror', err => console.log(`  [PAGE ERROR] ${err.message.substring(0, 200)}`));

  await page.setContent(html, { waitUntil: 'networkidle0', timeout: 60000 });

  // Wait for our MERMAID_DONE signal
  await page.waitForFunction(() => {
    return document.body.innerHTML.includes('<!-- mermaid-done -->') ||
           window.__mermaidDone === true;
  }, { timeout: 60000 }).catch(() => {});

  // Fallback: wait for all placeholders to have content
  await page.waitForFunction(() => {
    const placeholders = document.querySelectorAll('.mermaid-placeholder');
    return Array.from(placeholders).every(p => p.children.length > 0);
  }, { timeout: 30000 }).catch(() => {});

  await new Promise(r => setTimeout(r, 2000));

  console.log(`Generating PDF: ${outputFile}`);
  await page.pdf({
    path: outputFile,
    format: 'Letter',
    margin: { top: '0.75in', right: '0.75in', bottom: '0.75in', left: '0.75in' },
    printBackground: true,
    displayHeaderFooter: true,
    headerTemplate: '<span></span>',
    footerTemplate: '<div style="font-size:8pt;color:#999;width:100%;text-align:center;"><span class="pageNumber"></span> / <span class="totalPages"></span></div>',
  });

  await browser.close();
  console.log('Done!');
}

// CLI usage
const args = process.argv.slice(2);
if (args.length === 0) {
  console.log('Usage: node md-to-pdf.js <input.md> [output.pdf]');
  console.log('       node md-to-pdf.js --all  (converts all .md files)');
  process.exit(1);
}

(async () => {
  if (args[0] === '--all') {
    const mdFiles = fs.readdirSync('.').filter(f => f.endsWith('.md') && f !== 'README.md');
    for (const file of mdFiles) {
      const outFile = file.replace('.md', '.pdf');
      await convertMdToPdf(file, outFile);
    }
  } else {
    const input = args[0];
    const output = args[1] || input.replace('.md', '.pdf');
    await convertMdToPdf(input, output);
  }
})();
