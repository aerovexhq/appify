import DefaultTheme from 'vitepress/theme'
import './custom.css'
import LandingPage from './components/LandingPage.vue'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }: { app: any }) {
    app.component('LandingPage', LandingPage)
  }
}
