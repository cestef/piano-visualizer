import {
  Box,
  Button,
  ColorInput,
  Container,
  MantineProvider,
  Select,
  Slider,
  Text,
  Title,
} from '@mantine/core';
import { useEffect, useState } from 'react';

import http from './http';

const App = () => {
  const [colorMode, setColorMode] = useState('');
  const [solidColor, setSolidColor] = useState('#000000');
  const [animation, setAnimation] = useState('');
  const [brightness, setBrightness] = useState(0);
  useEffect(() => {
    http.get('/color_mode').then((response) => {
      switch (response.data.data) {
        case 'rainbow':
        case 'random':
          setColorMode(response.data.data);
          break;
        default:
          setColorMode('solid');
          setSolidColor(response.data.data);
          break;
      }
    });
    http.get('/animation').then((response) => {
      setAnimation(response.data.data);
    });
    http.get('/brightness').then((response) => {
      setBrightness(response.data.data);
    });
  }, []);
  return (
    <MantineProvider withNormalizeCSS withGlobalStyles>
      <Container mt='lg'>
        <Title>Piano LED Visualizer</Title>
        <Box mt='lg'>
          <Select
            label='Color Mode'
            data={[
              { label: 'Rainbow', value: 'rainbow' },
              { label: 'Random', value: 'random' },
              { label: 'Solid Color', value: 'solid' },
            ]}
            value={colorMode}
            onChange={(value) => {
              setColorMode(value as string);
              if (['rainbow', 'random'].includes(value as string)) {
                http.post('/color_mode', value);
              }
            }}
          />
          {colorMode === 'solid' && (
            <ColorInput
              swatches={[
                '#25262b',
                '#868e96',
                '#fa5252',
                '#e64980',
                '#be4bdb',
                '#7950f2',
                '#4c6ef5',
                '#228be6',
                '#15aabf',
                '#12b886',
                '#40c057',
                '#82c91e',
                '#fab005',
                '#fd7e14',
              ]}
              mt='md'
              value={solidColor}
              onChange={(value) => {
                setSolidColor(value);
              }}
              onBlur={() => {
                http.post('/color_mode', solidColor);
              }}
            />
          )}
        </Box>
        <Box mt='lg'>
          <Select
            label='Animation'
            data={[
              { label: 'Fade', value: 'fade' },
              { label: 'Ripple', value: 'ripple' },
              { label: 'None', value: 'none' },
              { label: 'Static Color', value: 'static' },
            ]}
            value={animation}
            onChange={(value) => {
              setAnimation(value as string);
              http.post('/animation', value);
            }}
          />
        </Box>
        <Box mt='lg'>
          <Text size='sm' weight={450} mb='xs'>
            Brightness
          </Text>
          <Slider
            size='lg'
            value={brightness}
            onChangeEnd={(e) => {
              http.post('/brightness', (e / 100) * 255);
            }}
            onChange={(value) => {
              setBrightness(value);
            }}
          />
        </Box>
        <Button
          onClick={() => {
            http.post('/animation', animation);
            if (colorMode === 'solid') {
              http.post('/color_mode', solidColor);
            } else {
              http.post('/color_mode', colorMode);
            }
          }}
        >
          Force Save All
        </Button>
      </Container>
    </MantineProvider>
  );
};

export default App;
