// Components module - uses the design system and i18n
import { init as initDesignSystem, VERSION as dsVersion } from '@xq9zk7823/design-system';
import { init as initI18n, VERSION as i18nVersion } from '@xq9zk7823/i18n-utils';

export function createApp(config) {
  const ds = initDesignSystem(config);
  const i18n = initI18n({ locale: config.locale || 'en' });

  return {
    designSystem: ds,
    i18n: i18n,
    versions: {
      designSystem: dsVersion,
      i18n: i18nVersion
    }
  };
}

export function renderWidget(name, props) {
  return {
    type: 'widget',
    name,
    props,
    rendered: true
  };
}
