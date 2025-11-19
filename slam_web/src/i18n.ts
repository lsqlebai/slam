export type Lang = 'zh' | 'en';

export const LANGUAGE_NAMES: Record<Lang, string> = {
  zh: '中文',
  en: 'English',
};

export const TEXTS = {
  zh: {
    login: {
      headTitle: '登录',
      title: '登录',
      username: '用户名',
      password: '密码',
      login: '登录',
      register: '注册',
      language: '语言',
    },
    register: {
      headTitle: '注册',
      title: '注册',
      username: '用户名',
      password: '密码',
      confirm: '确认密码',
      submit: '提交注册',
      cancel: '取消',
      language: '语言',
      errorFill: '请填写完整信息',
      errorLength: '密码长度至少6位',
      errorMismatch: '两次输入的密码不一致',
      success: '注册成功',
    },
  },
  en: {
    login: {
      headTitle: 'Login',
      title: 'Login',
      username: 'Username',
      password: 'Password',
      login: 'Login',
      register: 'Register',
      language: 'Language',
    },
    register: {
      headTitle: 'Register',
      title: 'Register',
      username: 'Username',
      password: 'Password',
      confirm: 'Confirm Password',
      submit: 'Submit',
      cancel: 'Cancel',
      language: 'Language',
      errorFill: 'Please fill in all fields',
      errorLength: 'Password must be at least 6 characters',
      errorMismatch: 'Passwords do not match',
      success: 'Registration successful',
    },
  },
} as const;

export function getSavedLang(): Lang {
  try {
    const saved = localStorage.getItem('lang');
    if (saved === 'zh' || saved === 'en') {
      return saved;
    }
  } catch {}
  return 'zh';
}

export function saveLang(lang: Lang): void {
  try {
    localStorage.setItem('lang', lang);
  } catch {}
}
