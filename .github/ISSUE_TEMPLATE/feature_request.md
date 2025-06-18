name: CLI Feature Request
description: Suggest a new idea for Nexus CLI.
body:
  - type: markdown
    attributes:
      value: |
        Thank you for using Nexus CLI!

        If you are looking for support, please check out our documentation
        or consider asking a question on [Discord](https://discord.gg/nexus-xyz).
  - type: textarea
    attributes:
      label: Use case
      description: |
        Please tell us the problem you are running into that led to you wanting
        a new feature.

        Is your feature request related to a problem? Please give a clear and
        concise description of what the problem is.

        Describe the alternative solutions you've considered.
    validations:
      required: true
  - type: textarea
    attributes:
      label: Proposal
      description: |
        Briefly but precisely describe what you would like Nexus CLI to be able to do.

        Consider attaching something showing what you are imagining:
         * images
         * videos
         * code samples
    validations:
      required: true
