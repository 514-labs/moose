export default {
    logo: <div>moosejs</div>,
    project: {
      link: 'https://github.com/514-labs/moose'
    },
    docsRepositoryBase: 'https://github.com/514-labs/moose/tree/main/apps/framework-docs',
    useNextSeoProps() {
        return {
          titleTemplate: '%s – MooseJS'
        }
      },
      primaryHue: 220,
      primarySaturation: 0,
      footer: {
        text: (
          <span>
            MIT | {new Date().getFullYear()} ©{' '} 
            <a href="https://fiveonefour.com" target="_blank">
                Fiveonefour Labs Inc
            </a>
            .
          </span>
        )
      }
  }

