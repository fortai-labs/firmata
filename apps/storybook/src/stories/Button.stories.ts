import { Button } from '@repo/ui/button';
import type { StoryObj } from "@storybook/react"
import type { Meta } from '@storybook/react';
 
const meta = {
  title: 'Example/Button',
  component: Button,
  tags: ['autodocs'],
} satisfies Meta<typeof Button>;
 
export default meta;
type Story = StoryObj<typeof meta>;
 
export const Primary: Story = {
  args: {
    appName: 'Button',
    children: 'I am a primary button.',
  },
};