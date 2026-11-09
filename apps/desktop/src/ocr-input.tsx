import type { Locale } from './content';
import type { useOcrInput } from './use-ocr-input';

export const ocrMessages = {
  zh: { image: '导入图片', preview: '截图预览', recognize: '识别文字', cancel: '取消图片', screen: '截图查词',
    hint: '导入 PNG／JPEG，或在输入框粘贴截图（Ctrl+V）。图片最多 16 MiB、4096×4096 像素；本地识别法语，结果最多保留前 5,000 字。',
    working: '正在本地识别…', failed: '图片或 OCR 处理失败：', empty: '没有识别到文字。请重新框选或导入清晰的图片。',
    uncertain: '识别结果不确定，请对照截图核对并编辑原文。', confidence: '识别置信度：' },
  en: { image: 'Import image', preview: 'Screenshot preview', recognize: 'Recognize text', cancel: 'Cancel image', screen: 'Screen lookup',
    hint: 'Import PNG/JPEG or paste a screenshot into the input (Ctrl+V). Maximum: 16 MiB, 4096×4096 pixels. French OCR runs locally and keeps the first 5,000 characters.',
    working: 'Recognizing locally…', failed: 'Image or OCR failed: ', empty: 'No text recognized. Select another region or import a clearer image.',
    uncertain: 'Recognition is uncertain. Compare the screenshot and edit the text.', confidence: 'Recognition confidence: ' },
  fr: { image: 'Importer une image', preview: 'Aperçu de la capture', recognize: 'Reconnaître le texte', cancel: 'Annuler l’image', screen: 'Rechercher par capture',
    hint: 'Importez un PNG/JPEG ou collez une capture dans le champ (Ctrl+V). Maximum : 16 Mio, 4096×4096 pixels. La reconnaissance locale du français garde les 5 000 premiers caractères.',
    working: 'Reconnaissance locale…', failed: 'Échec de l’image ou de la reconnaissance : ', empty: 'Aucun texte reconnu. Sélectionnez une autre zone ou importez une image plus nette.',
    uncertain: 'La reconnaissance est incertaine. Comparez la capture et corrigez le texte.', confidence: 'Confiance de reconnaissance : ' },
} as const;

export function OcrInput({ ocr, locale, onCancel }: {
  readonly ocr: ReturnType<typeof useOcrInput>; readonly locale: Locale; readonly onCancel: () => void;
}) {
  const text = ocrMessages[locale];
  return <div className="ocr-input">
    <button type="button" onClick={ocr.screen}>{text.screen}</button>
    <label>{text.image}<input type="file" accept="image/png,image/jpeg" onChange={(event) => {
      const file = event.target.files?.[0]; if (file) void ocr.importImage(file); event.target.value = '';
    }} /></label><p className="dictionary-hint">{text.hint}</p>
    {ocr.image && <><img className="ocr-preview" src={ocr.image.data_url} alt={text.preview} />
      <button type="button" disabled={ocr.busy} onClick={ocr.recognize}>{text.recognize}</button></>}
    {(ocr.image || ocr.busy) && <button type="button" onClick={() => { ocr.cancel(); onCancel(); }}>{text.cancel}</button>}
    {ocr.busy && <p role="status">{text.working} {Math.round(ocr.progress * 100)}%</p>}
    {ocr.error && <p role="alert">{text.failed}{ocr.error}</p>}
    {ocr.empty && <p role="alert">{text.empty}</p>}
    {ocr.confidence !== undefined && <p role="status">{text.confidence}{Math.round(ocr.confidence)}%</p>}
    {ocr.confidence !== undefined && ocr.confidence < 60 && <p role="alert">{text.uncertain}</p>}
  </div>;
}
