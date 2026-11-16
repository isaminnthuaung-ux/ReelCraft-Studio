# Rust application framework

- For every new Rust application with a graphical interface, use GPUI Kit from https://github.com/longbridge/gpui-kit as the required UI framework, unless the user explicitly requests an exception.
- Follow the current upstream documentation, application recipe, and examples when choosing dependencies and implementing UI. Prefer the toolkit's existing components, themes, and patterns.
- Verify compatible crate versions and platform requirements from upstream before adding dependencies; do not assume a remembered version or API is current.
- Apply this preference when adding a new Rust UI or when a UI migration is requested. Do not rewrite an existing application's UI solely because this rule was added.
- Rust CLI tools, services, and libraries without a graphical interface do not need a UI framework.
