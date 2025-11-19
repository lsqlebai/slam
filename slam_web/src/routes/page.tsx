import { Helmet } from '@modern-js/runtime/head';
import './index.css';

const Index = () => (
  <div className="container-box">
    <Helmet>
      <meta
        name="viewport"
        content="width=device-width, initial-scale=1, viewport-fit=cover"
      />
      <meta name="theme-color" content="#ffffff" />
      <link
        rel="icon"
        type="image/x-icon"
        href="https://lf3-static.bytednsdoc.com/obj/eden-cn/uhbfnupenuhf/favicon.ico"
      />
    </Helmet>
    <main>
      <div className="title">
        欢迎使用
        <img
          className="logo"
          src="https://lf3-static.bytednsdoc.com/obj/eden-cn/zq-uylkvT/ljhwZthlaukjlkulzlp/modern-js-logo.svg"
          alt="Modern.js Logo"
        />
        <p className="name">Modern.js</p>
      </div>
      <p className="description">
        可在手机上操作的 H5 页面，修改{' '}
        <code className="code">src/routes/page.tsx</code>
      </p>
      <div className="grid">
        <a
          href="https://modernjs.dev/guides/get-started/introduction.html"
          target="_blank"
          rel="noopener noreferrer"
          className="card"
        >
          <h2>
            指南
            <img
              className="arrow-right"
              src="https://lf3-static.bytednsdoc.com/obj/eden-cn/zq-uylkvT/ljhwZthlaukjlkulzlp/arrow-right.svg"
              alt="Guide"
            />
          </h2>
          <p>了解 Modern.js 的全部特性。</p>
        </a>
        <a
          href="https://modernjs.dev/tutorials/foundations/introduction.html"
          target="_blank"
          className="card"
          rel="noreferrer"
        >
          <h2>
            教程
            <img
              className="arrow-right"
              src="https://lf3-static.bytednsdoc.com/obj/eden-cn/zq-uylkvT/ljhwZthlaukjlkulzlp/arrow-right.svg"
              alt="Tutorials"
            />
          </h2>
          <p>从零开始构建你的第一个应用。</p>
        </a>
        <a
          href="https://modernjs.dev/configure/app/usage.html"
          target="_blank"
          className="card"
          rel="noreferrer"
        >
          <h2>
            配置
            <img
              className="arrow-right"
              src="https://lf3-static.bytednsdoc.com/obj/eden-cn/zq-uylkvT/ljhwZthlaukjlkulzlp/arrow-right.svg"
              alt="Config"
            />
          </h2>
          <p>查看 Modern.js 提供的全部配置项。</p>
        </a>
        <a
          href="https://github.com/web-infra-dev/modern.js"
          target="_blank"
          rel="noopener noreferrer"
          className="card"
        >
          <h2>
            GitHub
            <img
              className="arrow-right"
              src="https://lf3-static.bytednsdoc.com/obj/eden-cn/zq-uylkvT/ljhwZthlaukjlkulzlp/arrow-right.svg"
              alt="Github"
            />
          </h2>
          <p>查看源码并参与贡献。</p>
        </a>
      </div>
    </main>
  </div>
);

export default Index;
