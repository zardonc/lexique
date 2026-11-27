import type { Locale } from './content';

const messages: Record<Locale, Record<string, string>> = {
  zh: { unconfigured: '请先启用翻译服务。', credential_missing: '请先配置 API 密钥。', credential_unreadable: '无法读取密钥，请重新配置。', auth_failed: '认证失败，请检查密钥。', quota_or_rate_limited: '额度已用尽或请求过于频繁。', invalid_request: '请求无效，请检查文本和服务设置。', network_or_timeout: '网络连接失败或请求超时。', invalid_response: '服务返回了无效响应。', settings_error: '无法读取或保存翻译设置。', clipboard_error: '无法复制译文。', unknown: '操作失败，请检查服务设置。' },
  en: { unconfigured: 'Enable a translation service first.', credential_missing: 'Configure an API key first.', credential_unreadable: 'The key cannot be read; configure it again.', auth_failed: 'Authentication failed; check the key.', quota_or_rate_limited: 'Quota exhausted or too many requests.', invalid_request: 'Invalid request; check the text and service settings.', network_or_timeout: 'Network connection failed or timed out.', invalid_response: 'The service returned an invalid response.', settings_error: 'Could not read or save translation settings.', clipboard_error: 'Could not copy the translation.', unknown: 'The operation failed; check the service settings.' },
  fr: { unconfigured: 'Activez d’abord un service de traduction.', credential_missing: 'Configurez d’abord une clé API.', credential_unreadable: 'La clé est illisible ; configurez-la à nouveau.', auth_failed: 'Échec de l’authentification ; vérifiez la clé.', quota_or_rate_limited: 'Quota épuisé ou trop de requêtes.', invalid_request: 'Requête invalide ; vérifiez le texte et les paramètres.', network_or_timeout: 'Échec ou expiration de la connexion.', invalid_response: 'Le service a renvoyé une réponse invalide.', settings_error: 'Impossible de lire ou enregistrer les paramètres.', clipboard_error: 'Impossible de copier la traduction.', unknown: 'Échec de l’opération ; vérifiez les paramètres.' },
};

export function translationError(locale: Locale, error: unknown): string {
  const code = typeof error === 'string' ? error : error instanceof Error ? error.message : '';
  return messages[locale][code] ?? messages[locale].unknown;
}
