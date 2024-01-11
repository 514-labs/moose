export default {
    logo: <span>My Nextra Documentation</span>,
    project: {
      link: 'https://github.com/shuding/nextra'
    },
    useNextSeoProps() {
        return {
          titleTemplate: '%s – MooseJS'
        }
      },

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