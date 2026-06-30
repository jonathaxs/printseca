// Gera as páginas de manutenção do Printseca em DeviceCMYK.
// Usar CMYK (e não RGB) força cada tinta a ser depositada de fato:
//   ciano = (1,0,0,0), magenta = (0,1,0,0), amarelo = (0,0,1,0), preto = (0,0,0,1).
// Isso é o que mantém os bicos/cabeças desentupidos por uso periódico.
import { PDFDocument, StandardFonts, cmyk } from 'pdf-lib';
import { writeFileSync } from 'node:fs';

const W = 595.28, H = 841.89, M = 36; // A4 retrato, margem
const K = cmyk(0, 0, 0, 1);
const GREY = cmyk(0, 0, 0, 0.65);

function drawRow(page, font, items, x0, yTop, totalW, blockH) {
  const gap = 10;
  const n = items.length;
  const w = (totalW - gap * (n - 1)) / n;
  const yBlock = yTop - blockH;
  items.forEach((it, i) => {
    const x = x0 + i * (w + gap);
    page.drawRectangle({ x, y: yBlock, width: w, height: blockH, color: it.c });
    page.drawText(it.label, { x: x + 3, y: yBlock - 11, size: 8, font, color: K });
  });
  return yBlock - 22;
}

function drawRamp(page, font, name, mk, levels, x0, yTop, totalW, h) {
  page.drawText(name, { x: x0, y: yTop - h / 2 - 3, size: 9, font, color: K });
  const labelW = 22, gap = 4, n = levels.length;
  const w = (totalW - labelW - gap * (n - 1)) / n;
  const yBlock = yTop - h;
  levels.forEach((v, i) => {
    const x = x0 + labelW + i * (w + gap);
    page.drawRectangle({ x, y: yBlock, width: w, height: h, color: mk(v) });
  });
  return yBlock - 12;
}

async function build({ color }) {
  const doc = await PDFDocument.create();
  const page = doc.addPage([W, H]);
  const font = await doc.embedFont(StandardFonts.Helvetica);
  const bold = await doc.embedFont(StandardFonts.HelveticaBold);
  const contentW = W - 2 * M;

  let y = H - M - 16;
  page.drawText('PRINTSECA', { x: M, y, size: 24, font: bold, color: K });
  y -= 16;
  page.drawText(
    color ? 'Pagina de manutencao - exercita todas as tintas (CMYK)'
          : 'Pagina de manutencao - preto e branco',
    { x: M, y, size: 10, font, color: GREY },
  );
  y -= 26;

  if (color) {
    y = drawRow(page, font, [
      { label: 'CIANO',   c: cmyk(1, 0, 0, 0) },
      { label: 'MAGENTA', c: cmyk(0, 1, 0, 0) },
      { label: 'AMARELO', c: cmyk(0, 0, 1, 0) },
      { label: 'PRETO',   c: cmyk(0, 0, 0, 1) },
    ], M, y, contentW, 96);

    y = drawRow(page, font, [
      { label: 'VERMELHO', c: cmyk(0, 1, 1, 0) },
      { label: 'VERDE',    c: cmyk(1, 0, 1, 0) },
      { label: 'AZUL',     c: cmyk(1, 1, 0, 0) },
      { label: 'RICH BLACK', c: cmyk(0.6, 0.5, 0.5, 1) },
    ], M, y, contentW, 96);

    y -= 6;
    const levels = [1, 0.85, 0.7, 0.55, 0.4, 0.25, 0.12];
    const channels = [
      { name: 'C', mk: (v) => cmyk(v, 0, 0, 0) },
      { name: 'M', mk: (v) => cmyk(0, v, 0, 0) },
      { name: 'Y', mk: (v) => cmyk(0, 0, v, 0) },
      { name: 'K', mk: (v) => cmyk(0, 0, 0, v) },
    ];
    for (const ch of channels) y = drawRamp(page, font, ch.name, ch.mk, levels, M, y, contentW, 28);
  } else {
    const levels = [1, 0.9, 0.8, 0.7, 0.6, 0.5, 0.4, 0.3, 0.2, 0.1];
    y = drawRamp(page, font, 'K', (v) => cmyk(0, 0, 0, v), levels, M, y, contentW, 46);
    y -= 10;
    page.drawRectangle({ x: M, y: y - 150, width: contentW, height: 150, color: K });
    page.drawText('100% K', { x: M + 6, y: y - 16, size: 10, font, color: cmyk(0, 0, 0, 0) });
    y -= 162;
  }

  page.drawText('Imprima a cada ~7 a 14 dias para manter os bicos da impressora funcionando.',
    { x: M, y: M + 14, size: 9, font, color: GREY });
  page.drawText('Gerado por Printseca.', { x: M, y: M, size: 9, font, color: GREY });

  return await doc.save();
}

const outDir = process.argv[2] || '.';
writeFileSync(`${outDir}/manutencao-cor.pdf`, await build({ color: true }));
writeFileSync(`${outDir}/manutencao-pb.pdf`, await build({ color: false }));
console.log('PDFs gerados em', outDir);
