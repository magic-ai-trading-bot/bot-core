/**
 * Mock for echarts-for-react in test environments.
 * ECharts requires canvas APIs not available in jsdom.
 */
import React from 'react'

const ReactECharts = ({ style }: { style?: React.CSSProperties; [key: string]: unknown }) => (
  <div data-testid="echarts" style={style} />
)

export default ReactECharts
